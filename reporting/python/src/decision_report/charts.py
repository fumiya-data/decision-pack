from __future__ import annotations

from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib import font_manager

from .models import Report

_JAPANESE_FONT_CANDIDATES = [
    "Noto Sans JP",
    "Meiryo",
    "Yu Gothic",
    "BIZ UDGothic",
    "MS Gothic",
]


def _configure_japanese_font() -> None:
    available = {font.name for font in font_manager.fontManager.ttflist}
    for name in _JAPANESE_FONT_CANDIDATES:
        if name in available:
            plt.rcParams["font.family"] = name
            break
    plt.rcParams["axes.unicode_minus"] = False


def render_report(report: Report, out_dir: Path) -> None:
    _configure_japanese_font()
    out_dir.mkdir(parents=True, exist_ok=True)

    _plot_cash_balance(report, out_dir / "cash_balance.png")
    _plot_daily_stockout(report, out_dir / "daily_stockout.png")
    _write_summary(report, out_dir / "summary.txt")


def _plot_cash_balance(report: Report, out_path: Path) -> None:
    dates = [row["date"] for row in report.cash_series]
    cash = [row["cash"] for row in report.cash_series]

    fig, ax = plt.subplots(figsize=(9, 4.5))
    ax.plot(dates, cash, marker="o", linewidth=2)
    ax.set_title(f"資金残高 - {report.scenario.name}")
    ax.set_xlabel("日付")
    ax.set_ylabel(f"資金 ({report.currency})")
    ax.grid(alpha=0.3)
    fig.autofmt_xdate(rotation=30)
    fig.tight_layout()
    fig.savefig(out_path, dpi=140)
    plt.close(fig)


def _plot_daily_stockout(report: Report, out_path: Path) -> None:
    per_day = defaultdict(int)
    for row in report.inventory_series:
        per_day[row["date"]] += int(row["stockout"])

    dates = sorted(per_day.keys())
    values = [per_day[d] for d in dates]

    fig, ax = plt.subplots(figsize=(9, 4.5))
    ax.bar(dates, values)
    ax.set_title(f"日次欠品数量 - {report.scenario.name}")
    ax.set_xlabel("日付")
    ax.set_ylabel("欠品数量")
    ax.grid(alpha=0.3, axis="y")
    fig.autofmt_xdate(rotation=30)
    fig.tight_layout()
    fig.savefig(out_path, dpi=140)
    plt.close(fig)


def _write_summary(report: Report, out_path: Path) -> None:
    shortfall_date = report.kpi.first_cash_shortfall_date or "なし"
    lines = [
        f"スキーマ版: {report.schema_version}",
        f"生成時刻: {report.generated_at}",
        f"シナリオ: {report.scenario.id} ({report.scenario.name})",
        f"予測日数: {report.horizon_days}",
        f"最小資金残高: {report.kpi.min_cash}",
        f"初回資金ショート日: {shortfall_date}",
        f"総欠品数量: {report.kpi.total_stockout_qty}",
        f"欠品率: {report.kpi.stockout_rate}",
        f"平均在庫日数: {report.kpi.days_on_hand_avg}",
        f"アラート件数: {len(report.alerts)}",
    ]
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
