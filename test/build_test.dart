import 'dart:io';

import 'package:nucleo_dart/nucleo_dart.dart';

import 'package:test/test.dart';

void main() {
  test('FFI call works', () {
    final instance = NucleoDart(() {});
    instance.add(0, "entry1");
    instance.add(1, "entry2");
    instance.add(2, "entry3");

    instance.reparse("");
    instance.tick();
    var items = instance.getSnapshot().matchedItems().toList();
    expect(items, equals([(0, "entry1"), (1, "entry2"), (2, "entry3")]));

    instance.reparse("entry");
    instance.tick();
    items = instance.getSnapshot().matchedItems().toList();
    expect(items, equals([(0, "entry1"), (1, "entry2"), (2, "entry3")]));

    instance.reparse("entry2");
    instance.tick();
    items = instance.getSnapshot().matchedItems().toList();
    expect(items, equals([(1, "entry2")]));
  });

  test('Performance', () {
    final process = Process.runSync("fish", ["-c", "history"]);
    final data = process.stdout;
    final lines = data.split("\n");

    final sw = Stopwatch()..start();
    final instance = NucleoDart(() {});

    int i = 0;
    for (final line in lines) {
      instance.add(i, line);
      i++;
    }
    instance.reparse("some");
    instance.tick();

    print("ELAPSED ${instance.getSnapshot().matchedCount} ${sw.elapsed}");
  });

  test('Performance Async', () async {
    final process = Process.runSync("fish", ["-c", "history"]);
    final data = process.stdout;
    final lines = data.split("\n");

    final sw = Stopwatch()..start();
    final instance = NucleoDart(() {});

    await instance.addAllAsync(lines);

    // int i = 0;
    // for (final line in lines) {
    //   instance.add(i, line);
    //   i++;
    // }
    instance.reparse("some");
    instance.tick();

    print("ELAPSED ${instance.getSnapshot().matchedCount} ${sw.elapsed}");
  });
}
