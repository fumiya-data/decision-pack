from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Scenario:
    id: str
    name: str
    description: str | None


@dataclass(frozen=True)
class KPI:
    min_cash: int
    first_cash_shortfall_date: str | None
    total_stockout_qty: int
    stockout_rate: float
    days_on_hand_avg: float


@dataclass(frozen=True)
class Report:
    schema_version: str
    generated_at: str
    scenario: Scenario
    horizon_days: int
    currency: str
    kpi: KPI
    alerts: list[dict[str, Any]]
    cash_series: list[dict[str, Any]]
    inventory_series: list[dict[str, Any]]


def load_report(path: Path) -> Report:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if raw.get("schema_version") != "v0.1":
        raise ValueError("未対応の schema_version です。v0.1 を想定しています。")

    scenario = raw.get("scenario", {})
    kpi = raw.get("kpi", {})

    return Report(
        schema_version=raw["schema_version"],
        generated_at=raw["generated_at"],
        scenario=Scenario(
            id=scenario["id"],
            name=scenario["name"],
            description=scenario.get("description"),
        ),
        horizon_days=raw["horizon_days"],
        currency=raw["currency"],
        kpi=KPI(
            min_cash=kpi["min_cash"],
            first_cash_shortfall_date=kpi.get("first_cash_shortfall_date"),
            total_stockout_qty=kpi["total_stockout_qty"],
            stockout_rate=kpi["stockout_rate"],
            days_on_hand_avg=kpi["days_on_hand_avg"],
        ),
        alerts=raw.get("alerts", []),
        cash_series=raw.get("cash_series", []),
        inventory_series=raw.get("inventory_series", []),
    )
