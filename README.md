# rusimg

![Crates.io Version](https://img.shields.io/crates/v/rusimg)
[![Rust](https://github.com/yotiosoft/rusimg/actions/workflows/rust.yml/badge.svg)](https://github.com/yotiosoft/rusimg/actions/workflows/rust.yml)

A tool to run DeepL translations on command line written by Rust.

## Features

- Image Conversion (jpeg, png, webp, bmp)
- Set Conversion Quality
- Image Resizing
- Image Cropping
- Grayscale Conversion
- Save the image

### Image Conversion

Rusimg can convert images to the following formats.  

- For binary crates, the conversion format can be specified with the ``-c`` option.
- For library crates, the conversion format can be specified by calling the ``rusimg::RusImg.convert()`` function.

| format | binary crate option | library crate extension             |
| ------ | ------------------- | ----------------------------------- |
| jpeg   | -c jpeg             | Extension::Jpeg or Extension::Jpg * |
| png    | -c png              | Extension::Png                      |
| webp   | -c webp             | Extension::Webp                     |
| bmp    | -c bmp              | Extension::Bmp                      |

\* The ``rusimg::Extension::Jpeg`` and ``rusimg::Extension::Jpg`` are the same, but file names will be saved as ``.jpeg`` and ``.jpg`` respectively.


### Set Conversion Quality

Rusimg can set the quality of the converted image. This depends on each image format.

- For binary crates, the quality can be specified with the ``-q`` option. 
- For library crates, the quality can be specified by calling the ``rusimg::RusImg.compress()`` function.

| format | quality                                                      | note                                                         |
| ------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| jpeg   | 0-100                                                        | By default, the quality is set to 75.                        |
| png    | [0, 17.0], (17.0, 34.0], (34.0, 51.0], (51.0, 68.0], (68.0, 85.0], (85.0, 100.0] | Because the ``oxipng`` crate must be set to the 6 compression levels, input values will be converted into 6 levels. By default, the quality is set to 68.0-85.0. |
| webp   | 0-100                                                        | By default, the quality is set to 75.0.                      |
| bmp    | none                                                         | BMP does not have a quality setting because it is a lossless format. |

### Image Resizing

Resize images. The resize ratio is specified by a scaling factor (0, 100].

- For binary crates, the resize ratio can be specified with the ``-r`` option.
- For library crates, the resize ratio can be specified by calling the ``rusimg::RusImg.resize()`` function.

### Image Cropping

Crop images.

- For binary crates, the crop size can be specified with the ``-t`` option.
- For library crates, the crop size can be specified by calling the ``rusimg::RusImg.trim()`` or ``rusimg::RusImg.trim_rect()`` function.

### Grayscale Conversion

Convert images to grayscale.

- For binary crates, the grayscale conversion can be specified with the ``-g`` option.
- For library crates, the grayscale conversion can be specified by calling the ``rusimg::RusImg.grayscale()`` function.

### Save the image

Save the image to the specified file path.

## Binary crate

### Install

Use ``cargo`` to install the binary crate.

```bash
$ cargo install rusimg
```

The binary crates contain ``app`` features required to run the application by default, but this is not necessary when used as a library.

### Binary crate options

|option|description|
|--|--|
|-i, --input \<INPUT\>|Specify input file path. \<INPUT\> is the input file path. Multiple files and wildcards are supported. (e.g. *.jpg, *.png, *.webp, *.bmp)|
|-o, --output \<OUTPUT\>|Specify output directory or output file name. \<OUTPUT\> is the output directory or output file name.|
|-c, --convert \<CONVERT\>|Image Conversion（jpeg, png, webp, bmp）. \<CONVERT\> is the image format to convert to.|
|-r, --resize \<RESIZE\>|Image resizing (specified by scaling factor: (0, 100]). \<RESIZE\> is the scaling factor percentage.|
|-t, --trim \<TRIM\>|Image cropping. Input format: 'XxY+W+H' (e.g.100x100+50x50)|
|-g, --grayscale|Grayscale conversion.|
|-q, --quality \<QUALITY\>|Image quality. \<QUALITY\> is the image quality (0, 100].|
|-a, --append \<APPEND\>|Append a string to the file name. \<APPEND\> is the string to append. (e.g. -a "_new")|
|-d, --double-extension|Set output file path to double extension. (e.g. "image.jpg.webp")|
|-D, --delete|Delete the original file.|
|-y, --yes|If the destination file already exists, overwrite it without asking.|
|-n, --no|If the destination file already exists, do not overwrite it without asking.|
|-T, --threads \<THREADS\>|Number of threads to use. \<THREADS\> is the number of threads to use. Default: 4|
|-v, --view|View the image. Use ``viuer`` crate.|
|-h, --help|Display help message.|
|-V, --version|Display version information.|
|--recursive|Recursively process all files in the directory.|

## Library crate

See [librusimg](https://github.com/yotiosoft/librusimg) for the library crate information.
