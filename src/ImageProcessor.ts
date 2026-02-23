import NativeImageProcessor from './NativeImageProcessor';

export type FitMode = 'cover' | 'contain' | 'fill';
export type ImageFormat = 'jpeg' | 'png' | 'webp';
export type RotateDegrees = 90 | 180 | 270;

export interface ResizeOptions {
  width?: number;
  height?: number;
  fit?: FitMode;
}

export interface CompressOptions {
  quality?: number;
  format?: ImageFormat;
}

export interface CropOptions {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ImageResult {
  uri: string;
  width: number;
  height: number;
  format: string;
  size: number;
  compressionRatio: number;
}

class ImagePipeline {
  constructor(path: string) {
    NativeImageProcessor.setFilePath(path);
  }

  resize(options: ResizeOptions): this {
    if (!options.width && !options.height) {
      throw new Error('resize: at least one of width or height is required');
    }
    NativeImageProcessor.resize(
      options.width  ?? -1,
      options.height ?? -1,
      options.fit    ?? ''
    );
    return this;
  }

  compress(options: CompressOptions = {}): this {
    if (options.quality !== undefined && (options.quality < 0 || options.quality > 100)) {
      throw new Error('compress: quality must be between 0 and 100');
    }
    NativeImageProcessor.compress(
      options.quality ?? -1,
      options.format  ?? ''
    );
    return this;
  }

  crop(options: CropOptions): this {
    NativeImageProcessor.crop(options.x, options.y, options.width, options.height);
    return this;
  }

  rotate(degrees: RotateDegrees): this {
    NativeImageProcessor.rotate(degrees);
    return this;
  }

  flip(direction: 'horizontal' | 'vertical'): this {
    NativeImageProcessor.flip(direction === 'horizontal');
    return this;
  }

  save(): Promise<ImageResult> {
    return NativeImageProcessor.save();
  }
}

export const ImageProcessor = {
  load(path: string): ImagePipeline {
    if (!path) throw new Error('load: path is required');
    return new ImagePipeline(path);
  }
};
