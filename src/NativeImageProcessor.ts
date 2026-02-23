import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface ImageResult {
  uri: string;
  width: number;
  height: number;
  format: string;
  size: number;
  compressionRatio: number;
}

export interface ResizeOptions {
  width: number;
  height: number;
  fit: string;
}

export interface CompressOptions {
  quality: number;
  format: string;
  maxWidth: number;
  maxHeight: number;
}

interface Spec extends NativeModule {
  setFilePath(path: string): void;
  resize(width: number, height: number, fit: string): void;
  compress(quality: number, format: string): void;
  crop(x: number, y: number, width: number, height: number): void;
  rotate(degrees: number): void;
  flip(horizontal: boolean): void;
  save(): Promise<ImageResult>;
}

export default NativeModuleRegistry.getEnforcing<Spec>('ImageProcessor');
