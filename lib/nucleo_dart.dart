library;

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:isolate';
import 'dart:typed_data';
import 'package:ffi/ffi.dart';

import 'package:nucleo_dart/nucleo_dart_bindings.dart';

class NucleoDart implements Finalizable {
  static Pointer<NativeFunction<Void Function(Pointer<NucleoHandle>)>> _addressNucleoDartDestroy =
      Native.addressOf(nucleo_dart_destroy);
  static final _finalizeNucleo = NativeFinalizer(_addressNucleoDartDestroy.cast());

  static _relaseArena(Arena arena) => arena.releaseAll();
  static final _finalizeEntries = Finalizer(_relaseArena);

  static _closeCallback(NativeCallable<Void Function()> cb) => cb.close();
  static final _finalizeCallback = Finalizer(_closeCallback);

  late final Pointer<NucleoHandle> _handle;
  late final Arena arenaEntriesString;
  late NativeCallable<Void Function()> _notify;
  String? _prevText;

  NucleoDart(void Function() changeNotify) {
    arenaEntriesString = Arena();

    _notify = NativeCallable<Void Function()>.listener(changeNotify);
    _handle = nucleo_dart_new(_notify.nativeFunction);

    _finalizeNucleo.attach(this, Pointer.fromAddress(_handle.address), detach: this);
    _finalizeEntries.attach(this, arenaEntriesString, detach: this);
    _finalizeCallback.attach(this, _notify, detach: this);
  }

  void destroy() {
    _finalizeNucleo.detach(this);
    nucleo_dart_destroy(_handle);

    _finalizeEntries.detach(this);
    arenaEntriesString.releaseAll();

    _finalizeCallback.detach(this);
    _notify.close();
  }

  void add(int index, String entry) {
    final strNative = entry.toNativeUtf8(allocator: arenaEntriesString);

    using((arena) {
      final str = arena<NucleoDartStringMut>();
      str.ref.index = index;
      str.ref.len = strNative.length;
      str.ref.ptr = strNative.cast();

      nucleo_dart_add(_handle, str.ref);
    });
  }

  void addNative(int index, Pointer<Uint8> pointer, int length) {
    using((arena) {
      final str = arena<NucleoDartStringMut>();
      str.ref.index = index;
      str.ref.len = length;
      str.ref.ptr = pointer.cast();

      nucleo_dart_add(_handle, str.ref);
    });
  }

  static List<({int addr, int len})> _fromEntries(Arena arena, Iterable<Uint8List> entries) {
    int totalBytes = 0;
    for (final entry in entries) {
      totalBytes += entry.length;
    }
    Pointer<Uint8> listStart = arena<Uint8>(totalBytes);
    final response = <({int addr, int len})>[];

    for (final entry in entries) {
      response.add((addr: listStart.address, len: entry.length));
      listStart.asTypedList(entry.length).setRange(0, entry.length, entry);
      listStart = listStart + entry.length;
    }
    return response;
  }

  void addAll(Iterable<String> entries, [List<int>? entriesIndexes]) {
    final length = entries.length;

    final entriesNative = _fromEntries(arenaEntriesString, entries.map(utf8.encode));

    using((arena) {
      final list = arena<NucleoDartStringMut>(length);

      int i = 0;
      for (final entry in entriesNative) {
        final str = list + i;
        str.ref.index = entriesIndexes?[i] ?? i;
        str.ref.len = entry.len;
        str.ref.ptr = Pointer.fromAddress(entry.addr);

        i += 1;
      }

      nucleo_dart_add_all(_handle, list, length);
    });
  }

  void addAllNative(int index, List<(Pointer<Uint8>, int)> pointers, [List<int>? entriesIndexes]) {
    assert(entriesIndexes != null ? pointers.length == entriesIndexes.length : true);
    using((arena) {
      final list = arena<NucleoDartStringMut>(pointers.length);

      int i = 0;
      for (final entry in pointers) {
        final str = list + i;
        str.ref.index = entriesIndexes?[i] ?? i;
        str.ref.len = entry.$2;
        str.ref.ptr = entry.$1;

        i += 1;
      }

      nucleo_dart_add_all(_handle, list, pointers.length);
    });
  }

  Future<void> addAllAsync(List<String> entries, [List<int>? entriesIndexes]) async {
    using((arena) async {
      final result = Completer<int>();
      final resultPort = RawReceivePort();
      await Isolate.spawn(_buildEntriesAsync, (
        arenaLocal: arena,
        arenaEntries: arenaEntriesString,
        entries: entries,
        entriesIndexes: entriesIndexes,
        sp: resultPort.sendPort,
      ));

      resultPort.handler = (response) {
        result.complete(response as int);
      };
      final listAddr = await result.future;
      nucleo_dart_add_all(_handle, Pointer.fromAddress(listAddr), entries.length);
    });
  }

  static void _buildEntriesAsync(
    ({
      Arena arenaLocal,
      Arena arenaEntries,
      Iterable<String> entries,
      List<int>? entriesIndexes,
      SendPort sp,
    })
    params,
  ) {
    final length = params.entries.length;

    final entriesNative = _fromEntries(params.arenaEntries, params.entries.map(utf8.encode));

    final list = params.arenaLocal<NucleoDartStringMut>(length);

    int i = 0;
    for (final entry in entriesNative) {
      final str = list + i;
      str.ref.index = params.entriesIndexes?[i] ?? i;
      str.ref.len = entry.len;
      str.ref.ptr = Pointer.fromAddress(entry.addr);

      i += 1;
    }
    params.sp.send(list.address);
  }

  Snapshot getSnapshot() {
    final ptr = nucleo_dart_get_snapshot(_handle);
    return Snapshot._(ptr);
  }

  void reparse(String newText) {
    final append = _prevText != null && newText.startsWith(_prevText!);

    using((arena) {
      final strNative = newText.toNativeUtf8(allocator: arena);
      final str = arena<NucleoDartString>();
      str.ref.len = strNative.length;
      str.ref.ptr = strNative.cast();

      nucleo_dart_reparse(_handle, str.ref, append ? IsAppend.IsAppendYes : IsAppend.IsAppendNo);
    });
    _prevText = newText;
  }

  /// The main way to interact with the matcher, this should be called regularly
  /// (for example each time a frame is rendered).
  ///
  /// To avoid excessive redraws this method will wait timeout milliseconds for the worker
  /// therad to finish.
  ///
  /// It is recommend to set the timeout to 10ms.
  void tick([int timoutMs = 10]) {
    nucleo_dart_tick(_handle, timoutMs);
  }
}

class Snapshot {
  final Pointer<SnapshotHandle> _handle;

  Snapshot._(this._handle);

  (int, String) item(int index) {
    assert(index >= 0);
    return nucleo_dart_snapshot_get_item(_handle, index).toDartString();
  }

  (int, String) matchedItem(int index) {
    assert(index >= 0);
    return nucleo_dart_snapshot_get_matched_item(_handle, index).toDartString();
  }

  int matchedItemIndex(int index) {
    assert(index >= 0);
    return nucleo_dart_snapshot_get_matched_item(_handle, index).index;
  }

  List<(int, String)> matchedItems([int start = 0, int? end]) {
    end ??= matchedCount;
    final response = <(int, String)>[];

    final callback = NativeCallable<Void Function(NucleoDartString)>.isolateLocal((
      NucleoDartString v,
    ) {
      response.add(v.toDartString());
    });

    nucleo_dart_snapshot_get_matched_items(_handle, start, end, callback.nativeFunction);
    callback.close();
    return response;
  }

  List<int> matchedItemsIndex([int start = 0, int? end]) {
    end ??= matchedCount;
    final response = <int>[];

    final callback = NativeCallable<Void Function(NucleoDartString)>.isolateLocal((
      NucleoDartString v,
    ) {
      response.add(v.index);
    });

    nucleo_dart_snapshot_get_matched_items(_handle, start, end, callback.nativeFunction);
    callback.close();
    return response;
  }

  int get matchedCount {
    return nucleo_dart_snapshot_get_matched_item_count(_handle);
  }

  int get count {
    return nucleo_dart_snapshot_get_item_count(_handle);
  }
}

extension on NucleoDartString {
  (int, String) toDartString() {
    Pointer<Utf8> nativeStrUtf8 = this.ptr.cast();
    return (this.index, nativeStrUtf8.toDartString(length: this.len));
  }
}
