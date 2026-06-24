#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

TMPDIR_FOR_VERIFY="$TMPDIR/.vunzip"
mkdir "$TMPDIR_FOR_VERIFY"

abort_verify() {
  ui_print "*********************************************************"
  ui_print "! $1"
  ui_print "! This zip may be corrupted, please try downloading again"
  abort    "*********************************************************"
}

# do_extract <junk> <zip> <file> <target dir> [name]
do_extract() {
  junk=$1
  zip=$2
  file=$3
  dir=$4
  name=$5
  opts="-o"
  if [ "$junk" = true ]; then
    opts="-oj"
  fi

  file_path=""
  hash_path=""
  if [ "$junk" = true ]; then
    file_path="$dir/$(basename "$file")"
    hash_path="$TMPDIR_FOR_VERIFY/$(basename "$file").sha256"
  else
    file_path="$dir/$file"
    hash_path="$TMPDIR_FOR_VERIFY/$file.sha256"
  fi

  unzip $opts "$zip" "$file" -d "$dir" >&2
  [ -f "$file_path" ] || abort_verify "$file not exists"

  unzip $opts "$zip" "$file.sha256" -d "$TMPDIR_FOR_VERIFY" >&2
  [ -f "$hash_path" ] || abort_verify "$file.sha256 not exists"

  (echo "$(cat "$hash_path")  $file_path" | sha256sum -c -s -) || abort_verify "Failed to verify $file"

  if [ -n "$name" ]; then
    real_path="$(dirname "$file_path")/$name"
    mv "$file_path" "$real_path" || abort "failed to rename $file_path to $name"
    ui_print "- extract $file -> $real_path" >&1
  else
    ui_print "- extract $file -> $file_path" >&1
  fi
}

# extract <path-in-zip> [out dir in MODPATH] [name]
extract() {
  if [ -n "$2" ]; then
    junk="true"
    out="$MODPATH/$2"
  else
    junk="false"
    out="$MODPATH"
  fi
  do_extract "$junk" "$ZIPFILE" "$1" "$out" "$3"
}
