import React, { useState } from 'react';
import { View, Image, Text, Platform, TouchableOpacity } from 'react-native';
import RNFS from 'react-native-fs';
import { ImageProcessor } from 'react-native-ferropix';


export default function App() {
  const [uri, setUri] = useState<string | null>(null);
  const [info, setInfo] = useState('');
  const [loading, setLoading] = useState(true);

  const getFilePath = async () => {
      let filePath: string;
    if (Platform.OS === 'android') {
      filePath = `${RNFS.DocumentDirectoryPath}/triangle.jpg`;
      const exists = await RNFS.exists(filePath);
      if (!exists) {
        console.log('called');
        await RNFS.copyFileAssets('triangle.jpg', filePath);
      }
    } else {
      filePath = `${RNFS.MainBundlePath}/triangle.jpg`;
    }
    return filePath;
  }

  const resize = async () => {
    setLoading(true);
    try {
      let filePath = await getFilePath();
      // ImageProcessor.load(filePath);
      console.log('filePath', filePath)
          const result = await ImageProcessor.load(filePath).crop({width : 271 , height : 187, x : 0, y : 22}).rotate(90).save();

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
          source={{ uri }}
          style={{ width: 400, height: 400,backgroundColor : 'red' }}
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
