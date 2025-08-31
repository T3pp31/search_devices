# Search Devices - Ping Scanner GUI

このアプリケーションは、RustとFLTKを使って作られた「ネットワーク到達性確認ツール（GUI）」です。指定したネットワーク（CIDR形式）やIPアドレスリストに対して、各ホストが生きているか（ping応答）を手軽に確認できます。さらに、経路の確認（traceroute/tracert）タブも追加されました。

ネットワーク管理やトラブルシューティングに最適です。

---

- 大きなネットワークをスキャンする場合は、PCの負荷やネットワークへの影響に注意しましょう。
- CIDR形式（例: 192.168.1.0/24）を使うと、同じネットワーク内の複数ホストを一度にチェックできます。
- IPリストタブでは、特定のホストだけをピンポイントで調べたいときに便利です。
- スキャン結果はリアルタイムで表示されるので、進捗が一目で分かります。


## 概要

RustとFLTKによるシンプルなGUIアプリケーションで、以下を提供します。
- CIDR/リストに対するICMP到達性スキャン（Ping）
- Ping設定の可変化（Count/Timeout）
- 経路確認（Tracert/Traceroute）タブ

## .exeファイルのダウンロード

[こちらから最新のリリース版をダウンロード](https://github.com/T3pp31/search_devices/releases)してください．

## 操作手順

### CIDRタブ

1. ネットワークをCIDR形式で入力（例: `192.168.1.0/24`）
2. Ping設定を必要に応じて調整
   - Count: 送信回数（既定: 1）
   - Timeout(ms): タイムアウト（既定: 1000ms）
     - Linux/Unixでは`ping -W`の仕様により秒へ切り上げ変換されます
3. 「Scan」でスキャン開始、結果はテキスト表示欄に追記されます。
4. 「Stop」で途中停止、「Clear」で結果をクリアします。

### IP Listタブ

1. スキャンしたいIPアドレスを1行ずつ入力（例: `192.168.1.10`）
2. Ping設定を必要に応じて調整（Count/TimeoutはCIDRタブと同じ仕様）
3. 「Scan List」でスキャン開始、結果はテキスト表示欄に追記されます。
4. 「Stop」で途中停止、「Clear」で入力と結果をクリアします。

### Tracertタブ（経路確認）

1. Target（IPまたはホスト名）を入力（例: `8.8.8.8`）
2. オプションを必要に応じて調整
   - Max Hops: 最大ホップ数（既定: 30）
   - Timeout(ms): タイムアウト（既定: 1000ms）
     - Linux/Unixの`traceroute -w`は秒指定のため、ミリ秒から切り上げ秒に変換されます
   - Resolve DNS: 逆引きを有効/無効化（無効化で高速化）
3. 「Trace」で実行、結果は逐次テキスト表示欄に表示されます（空行や前後空白は除去して表示）。
4. 「Stop」で実行中のプロセスを停止し、出力も停止します。

## 注意事項

- 大規模ネットワークではスキャン時間やリソース消費が多くなるためご注意ください。

- リリースビルドではバックグラウンド起動し、黒いコンソール画面は表示されません。

- Windows以外の環境ではコンソールが表示される場合があります。

- 本ツールはOSのコマンドを利用します。
  - Ping: `ping`
  - 経路確認: Windowsは`tracert`、Linux/Unixは`traceroute`
  - Linux/Unixで`traceroute`が未導入の場合は、パッケージマネージャでインストールしてください（例: `sudo apt install traceroute`）。
  - Linuxの`ping`/`traceroute`は権限やケーパビリティに依存する場合があります。

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

## テスト

ユニットテストを用意しています（GUIやネットワークに依存せず実行可能）。

```
cargo test
```

主なテスト内容:
- ms→sec切り上げ変換、出力行のサニタイズ
- OS別のping/traceroute引数の組み立て検証

## ライセンス

MIT
