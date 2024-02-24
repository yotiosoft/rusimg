# rusimg

A tool to run DeepL translations on command line written by Rust.

## Install

```bash
$ cargo install rusimg
```

## Binary
### Options

|オプション|内容|
|--|--|
|-o, --output|Specify output directory or output file name.|
|-c, --convert|Image Conversion.（jpeg, png, webp, bmp）|
|-r, --resize|Image resizing. (specified by scaling factor: (0, 100])|
|-t, --trim|Image cropping.|

### Image Conversion: options

|形式|オプション|
|--|--|
|jpeg|-c jpeg|
|png|-c png|
|webp|-c webp|
|bmp|-c bmp|

   


