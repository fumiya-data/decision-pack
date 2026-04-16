# doc-gen4 の推奨手順は「docbuild という入れ子プロジェクトを作る」方法

  このリポジトリ (name = "spec", [[lean_lib]] name = "Spec") だと次のとおりです。

  ルートで docbuild を作成

  mkdir docbuild

  docbuild/lakefile.toml を作成

  name = "docbuild"
  reservoir = false
  version = "0.1.0"
  packagesDir = "../.lake/packages"

  [[require]]
  name = "spec"
  path = "../"

  [[require]]
  scope = "leanprover"
  name = "doc-gen4"
  rev = "main"

1. 依存を更新
  cd docbuild
  lake update doc-gen4
  lake update spec

2. HTML 生成

  lake build Spec:docs

3. ブラウザ表示（直接 index.html を開かず、HTTP 経由）

```PowerShell
cd .lake/build/doc
python -m http.server 8000
```

- 開く: http://localhost:8000/index.html

  補足:

- DecisionSpec.lean の /-- ... -/ は生成ページに反映されます。
- VS Code へのローカルリンクにしたい場合は DOCGEN_SRC=vscode lake build Spec:docs も使えます。

参考: https://github.com/leanprover/doc-gen4