#include <stdint.h>

typedef enum IsAppend {
  IsAppendYes,
  IsAppendNo,
} IsAppend;

typedef struct NucleoHandle NucleoHandle;

typedef struct SnapshotHandle SnapshotHandle;

typedef void (*VoidCallbackFn)(void);

typedef struct NucleoDartStringMut {
  uint32_t index;
  uint8_t *ptr;
  uintptr_t len;
} NucleoDartStringMut;

typedef struct NucleoDartString {
  uint32_t index;
  const uint8_t *ptr;
  uintptr_t len;
} NucleoDartString;

typedef void (*AppendCallbackFn)(struct NucleoDartString);

struct NucleoHandle *nucleo_dart_new(VoidCallbackFn cb);

void nucleo_dart_destroy(struct NucleoHandle *ptr);

void nucleo_dart_tick(struct NucleoHandle *ptr, unsigned int ms);

void nucleo_dart_add(struct NucleoHandle *ptr, struct NucleoDartStringMut item);

void nucleo_dart_add_all(struct NucleoHandle *ptr,
                         const struct NucleoDartStringMut *list,
                         uintptr_t len);

/**
 * By specifying append the caller promises that text passed to the previous reparse invocation
 * is a prefix of new_text. This enables additional optimizations but can lead to missing matches
 * if an incorrect value is passed.
 */
void nucleo_dart_reparse(struct NucleoHandle *ptr,
                         struct NucleoDartString new_text,
                         enum IsAppend append);

const struct SnapshotHandle *nucleo_dart_get_snapshot(struct NucleoHandle *ptr);

uint32_t nucleo_dart_snapshot_get_item_count(const struct SnapshotHandle *handle);

uint32_t nucleo_dart_snapshot_get_matched_item_count(const struct SnapshotHandle *handle);

struct NucleoDartString nucleo_dart_snapshot_get_item(const struct SnapshotHandle *handle,
                                                      uint32_t index);

struct NucleoDartString nucleo_dart_snapshot_get_matched_item(const struct SnapshotHandle *handle,
                                                              uint32_t index);

void nucleo_dart_snapshot_get_matched_items(const struct SnapshotHandle *handle,
                                            uint32_t start,
                                            uint32_t end,
                                            AppendCallbackFn cb);
