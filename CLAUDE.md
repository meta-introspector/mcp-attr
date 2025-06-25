# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mcp-attrは、Model Context Protocol (MCP) サーバーを宣言的に構築するためのRustライブラリです。属性マクロを使用してMCPサーバーを簡単に作成できるように設計されています。

### Architecture

このプロジェクトは3つのワークスペースメンバーで構成されています：

- **mcp-attr**: メインライブラリ。MCPサーバーとクライアントの実装
- **mcp-attr-macros**: プロシージャルマクロの実装 (`#[mcp_server]`, `#[tool]`, `#[resource]`, `#[prompt]`)
- **codegen**: スキーマ生成とコード生成用のユーティリティ

### Key Components

- `#[mcp_server]` 属性によるMCPサーバーの宣言的記述
- `#[tool]`, `#[resource]`, `#[prompt]` 属性による機能の実装
- MCPクライアント（テスト用）
- 型システムを活用したスキーマ生成

## Development Commands

### Build and Test
```bash
# 全パッケージのビルド
cargo build

# 特定パッケージのビルド
cargo build -p mcp-attr
cargo build -p mcp-attr-macros
cargo build -p mcp-attr-codegen

# テスト実行
cargo test

# 特定パッケージのテスト
cargo test -p mcp-attr

# ドキュメントテスト
cargo test --doc

# コンパイル失敗テスト（trybuild）
cargo test compile_fail
```

### Code Quality
```bash
# 型チェック
cargo check

# テストのコンパイルチェック（実行なし）
cargo test --no-run

# Clippy（リンター）
cargo clippy

# 自動修正
cargo clippy --fix --allow-dirty

# ドキュメント生成
cargo doc

# フォーマット
cargo fmt
```

### Examples
```bash
# サンプル実行
cargo run --example char_count
cargo run --example tool_info
```

## Testing Strategy

### Test Organization
- `tests/` ディレクトリ: 統合テスト
- `tests/mcp_server_*.rs`: `#[mcp_server]` 属性のテスト（型ごとに1ファイル）
- `tests/compile_fail/`: コンパイル失敗テスト（trybuild使用）
- モジュール内 `tests` モジュール: 非公開項目のテスト

### Test Guidelines
- 新機能実装時は必ずテストを作成
- 複数テスト追加時は1つずつ追加して確認
- テストデータは英語を使用（非ASCII文字テスト時を除く）

### Debugging `#[mcp_server]` Macro
マクロのデバッグ時：
1. `#[mcp_server]` を `#[mcp_server(dump)]` に変更
2. テスト実行でマクロ展開後コードを確認
3. 展開後コードを直接編集してデバッグ
4. 修正内容をマクロ実装に反映

## Code Style

### Rust Conventions
- Rustの慣例とベストプラクティスに従う
- 関数名・型名は一貫性と対称性を重視
- 理解困難なコードのみにコメント付与
- バグ以外でErrが返されない場合はResultを使わずパニック

### Error Handling
- `mcp_attr::Result` と `mcp_attr::Error` を使用
- `bail!` (プライベート) と `bail_public!` (パブリック) マクロを活用
- 依存関係のエラーはプライベート情報として扱う

### Documentation
- 公開項目には適切なdocコメントを付与
- 最初の行は簡潔な1行説明
- 関連する型・関数は `[]` でリンク
- `cargo test --doc` と `cargo doc` で検証

## Dependencies

### Main Dependencies
- `serde`: JSON シリアライゼーション
- `tokio`: 非同期ランタイム
- `schemars`: JSON Schema生成
- `jsoncall`: JSON-RPC実装
- `uri-template-ex`: URI Template処理

### Development Dependencies
- `trybuild`: コンパイル失敗テスト
- `pretty_assertions`: テストアサーション

## Important Notes

- 依存関係のバージョンダウンは禁止
- カレントディレクトリ変更は避け、コマンド引数で対応
- エラー修正3回失敗時はスキップして他の箇所を修正
- 依存関係の追加・変更は禁止とし、必要な場合はユーザーによる手動編集を促すこと