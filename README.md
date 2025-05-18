# Search Devices - Ping Scanner GUI

## 概要

RustとFLTKによるシンプルなGUIアプリケーションで、指定したホストやネットワーク内の生存ホストをpingでスキャンします。

## .exeファイルのダウンロード

[こちらから最新のリリース版をダウンロード](https://github.com/T3pp31/search_devices/releases)してください．

## 操作手順

### CIDRタブ

1. ネットワークをCIDR形式で入力（例: `192.168.1.0/24`）

2. 「Scan」ボタンを押すと、スキャン結果がテーブル形式で表示されます。

3. 「Stop」ボタンで途中停止し、「Clear」ボタンで結果をクリアできます。

### IP Listタブ

1. スキャンしたいIPアドレスを1行ずつ入力（例: `192.168.1.10`）

2. 「Scan List」ボタンを押すと、リスト内の各IPを順次スキャンします。

3. 「Stop」ボタンで途中停止し、「Clear」ボタンで入力と結果をクリアできます。

## 注意事項

- 大規模ネットワークではスキャン時間やリソース消費が多くなるためご注意ください。

- リリースビルドではバックグラウンド起動し、黒いコンソール画面は表示されません。

- Windows以外の環境ではコンソールが表示される場合があります。

## ビルド方法

1. Rustをインストール（[rustup.rs](https://rustup.rs/)）

2. リポジトリをクローン

   ```powershell
   git clone https://github.com/yourname/search_devices.git
   ```

3. 依存ライブラリを取得してリリースビルド

   ```powershell
   cd search_devices
   cargo build --release
   ```

## 実行方法

- コマンドラインから:

  ```powershell
  cargo run --release
  ```

- Windowsの場合は、`target\release\search_devices.exe` をダブルクリックして起動できます。

## ライセンス

MIT
