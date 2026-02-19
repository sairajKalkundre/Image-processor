import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface ImageResult {
  base64: string;
  width: number;
  height: number;
  format: string;
}

interface Spec extends NativeModule {
  loadImage(path: string): Promise<ImageResult>;
}

export default NativeModuleRegistry.getEnforcing<Spec>('ImageProcessor');
