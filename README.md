99行パストレのNextEventEstimate版であるexplicit.cppをrustに写経実装中  
ご本家の版だと他のシーンモデルが動かないので球内部に視点がある場合の輝度計算を追加  
64サンプリングでも結構キレイになる。 
オプションは【 -s サンプリング数 -m　シーンモデル（0..9) -w 横幅 -o 出力画像ファイル 】  
  
![128サンプリングの絵](https://github.com/garbagememo/smallpt-nee-rust/blob/main/image.png)
  
同様に各種シーンを128サンプリングで空画像    
![空画像](https://github.com/garbagememo/smallpt-nee-rust/blob/main/sky.png)
    
謎画像  　　
![謎画像](https://github.com/garbagememo/smallpt-nee-rust/blob/main/wada.png)
  
  
島画像  
![島画像](https://github.com/garbagememo/smallpt-nee-rust/blob/main/island.png)  
  
