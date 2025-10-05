import 'dart:io';

import 'package:hooks/hooks.dart';
import 'package:code_assets/code_assets.dart';

void main(List<String> args) async {
  try {
    await build(args, (input, output) async {
      if (input.config.buildCodeAssets) {
        final libName = input.config.code.targetOS.libraryFileName(
          "nucleo_dart",
          DynamicLoadingBundled(),
        );
        final libUri = input.outputDirectory.resolve(libName);

        final result = await Process.run("cargo", [
          "build",
          // "--release",
        ], workingDirectory: input.packageRoot.resolve("rust").path);
        if (result.exitCode != 0) {
          throw "cargo command exitCode ${result.exitCode}";
        }

        final file = File(input.packageRoot.resolve("rust/target/release/$libName").path);
        file.copy(libUri.path);

        output.dependencies.add(input.packageRoot.resolve("rust/src/lib.rs"));
        output.assets.code.add(
          CodeAsset(
            package: input.packageName,
            name: 'nucleo_dart_bindings.dart',
            linkMode: DynamicLoadingBundled(),
            file: libUri,
          ),
          routing: ToAppBundle(),
        );
      }
    });
  } catch (e, st) {
    // ignore: avoid_print
    print("$e\n\n$st");
    exit(1);
  }
}
