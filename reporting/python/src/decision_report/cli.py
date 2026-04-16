from __future__ import annotations

import argparse
from pathlib import Path

from .charts import render_report
from .models import load_report


def main() -> None:
    parser = argparse.ArgumentParser(description="シミュレーションレポート JSON から図表を生成します")
    parser.add_argument("--input", required=True, type=Path, help="simulation_report_v0.1 JSON のパス")
    parser.add_argument("--out-dir", required=True, type=Path, help="図表ファイルの出力先ディレクトリ")
    args = parser.parse_args()

    report = load_report(args.input)
    render_report(report, args.out_dir)

    print(f"レポート成果物を生成しました: {args.out_dir}")


if __name__ == "__main__":
    main()
