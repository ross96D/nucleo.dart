import 'dart:io';

import 'package:nucleo_dart/nucleo_dart.dart';

import 'package:test/test.dart';

void main() {
  test('FFI call works', () {
    final instance = NucleoDart(() {});
    instance.add("entry1");
    instance.add("entry2");
    instance.add("entry3");

    instance.reparse("entry");
    instance.tick();
    final items = instance.getSnapshot().matchedItems().toList();
    expect(["entry1", "entry2", "entry3"], equals(items));

    instance.reparse("entry2");
    instance.tick();
    final items2 = instance.getSnapshot().matchedItems().toList();
    expect(["entry2"], equals(items2));
  });

  test('Performance', () {
    final data = File("/home/ross/programming/my/nucleo.dart/test.txt").readAsStringSync();
    final lines = data.split("\n");

    final sw = Stopwatch()..start();
    final instance = NucleoDart(() {});

    for (final line in lines) {
      instance.add(line);
    }
    instance.reparse("some");
    instance.tick();

    print("ELAPSED ${instance.getSnapshot().matchedCount} ${sw.elapsed}");
  });
}
