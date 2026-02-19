import React, { useEffect, useState } from 'react';
import { View, Image, Text, Platform } from 'react-native';
import RNFS from 'react-native-fs';
import { ImageProcessor } from 'Image-processor';

export default function App() {
  const [uri, setUri] = useState<string | null>(null);
  const [info, setInfo] = useState('');

  useEffect(() => {
    async function load() {
      try {
        let filePath: string;

        if (Platform.OS === 'android') {
          filePath = `${RNFS.DocumentDirectoryPath}/icon.png`;
          const exists = await RNFS.exists(filePath);
          if (!exists) {
            // 'icon.png' here is relative to your android/app/src/main/assets/
            await RNFS.copyFileAssets('icon.png', filePath);
          }
        } else {
          filePath = `${RNFS.MainBundlePath}/icon.png`;
        }
        console.log('filePath', filePath);
        const result = await ImageProcessor.loadImage(filePath);
        console.log('result', JSON.stringify(result));
        setUri(`data:image/${result.format};base64,${result.base64}`);
        setInfo(`${result.width}x${result.height}`);
      } catch (e) {
        console.log('Error loading image', e);
        setInfo(`Error: ${e}`);
      }
    }

    load();
  }, []);

  return (
    <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
      <Text>Loaded via Rust 🦀</Text>
      {uri && <Image source={{ uri }} style={{ width: 200, height: 200 }} />}
      <Text>{info}</Text>
    </View>
  );
}
