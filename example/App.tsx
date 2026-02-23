import React, { useState } from 'react';
import { View, Image, Text, Platform, TouchableOpacity } from 'react-native';
import RNFS from 'react-native-fs';
import { ImageProcessor } from 'Image-processor';


export default function App() {
  const [uri, setUri] = useState<string | null>(null);
  const [info, setInfo] = useState('');
  const [loading, setLoading] = useState(true);

  const getFilePath = async () => {
      let filePath: string;
    if (Platform.OS === 'android') {
      filePath = `${RNFS.DocumentDirectoryPath}/50mb.jpg`;
      const exists = await RNFS.exists(filePath);
      if (!exists) {
        console.log('called');
        await RNFS.copyFileAssets('50mb.jpg', filePath);
      }
    } else {
      filePath = `${RNFS.MainBundlePath}/potrait.jpg`;
    }
    return filePath;
  }

  const resize = async () => {
    setLoading(true);
    try {
      let filePath = await getFilePath();
      // ImageProcessor.load(filePath);
      console.log('filePath', filePath)
          const result = await ImageProcessor.load(filePath).resize({width : 750 , height : 450,fit : 'contain'}).save();
        console.log('resize', result);
      setUri(result.uri);
      setInfo(`${result.width}x${result.height}`);
    }

      catch (e) {
        console.log('Error loading image', e);
        setInfo(`Error: ${e}`);
      } finally {
        setLoading(false);
      }
  }

  return (
    <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
      <Text>Loaded via Rust 🦀</Text>
      {loading && <Text>Loading...</Text>}
      {!loading && uri && (
        <Image
          key={uri}
          source={{ uri }}
          style={{ width: 400, height: 200,backgroundColor : 'red' }}
          onLoad={() => console.log('Image rendered successfully')}
          onError={e => console.log('Image error:', e.nativeEvent.error)}
        />
      )}
      <Text>{info}</Text>

      <TouchableOpacity onPress={resize} style={{ marginTop: 20 ,height : 40 , width : 100, backgroundColor : 'red'}}>
        <Text>Resize</Text>
      </TouchableOpacity>
    </View>
  );
}
