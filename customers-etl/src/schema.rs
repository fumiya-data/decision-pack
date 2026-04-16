//! パーサ、フォーマッタ、レポートで共有する固定スキーマ定義です。

/// 顧客エクスポート各列の論理識別子です。
///
/// 生の添字ではなく enum を使うことで、分配処理を明示的に保ち、
/// 特定列の規則や集計値を扱うときの off-by-one ミスを防ぎます。
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(usize)]
pub enum Column {
    CustomerId = 0,
    FullName = 1,
    Email = 2,
    Phone = 3,
    AddressLine = 4,
    City = 5,
    Region = 6,
    PostalCode = 7,
    Country = 8,
    BirthDate = 9,
    SignupDate = 10,
    LastPurchaseDate = 11,
    Status = 12,
    Tier = 13,
    PreferredLanguage = 14,
    MarketingOptIn = 15,
    TotalSpend = 16,
    OrderCount = 17,
    Notes = 18,
}

impl Column {
    /// 入力が期待し、整形済み出力もこの順で書く全列一覧です。
    pub const ALL: [Column; 19] = [
        Column::CustomerId,
        Column::FullName,
        Column::Email,
        Column::Phone,
        Column::AddressLine,
        Column::City,
        Column::Region,
        Column::PostalCode,
        Column::Country,
        Column::BirthDate,
        Column::SignupDate,
        Column::LastPurchaseDate,
        Column::Status,
        Column::Tier,
        Column::PreferredLanguage,
        Column::MarketingOptIn,
        Column::TotalSpend,
        Column::OrderCount,
        Column::Notes,
    ];

    /// この列の正規 CSV ヘッダ名です。
    pub const fn header(self) -> &'static str {
        match self {
            Column::CustomerId => "CustomerID",
            Column::FullName => "full_name",
            Column::Email => "email",
            Column::Phone => "phone",
            Column::AddressLine => "address_line",
            Column::City => "city",
            Column::Region => "region",
            Column::PostalCode => "postal_code",
            Column::Country => "country",
            Column::BirthDate => "birth_date",
            Column::SignupDate => "signup_date",
            Column::LastPurchaseDate => "last_purchase_date",
            Column::Status => "status",
            Column::Tier => "tier",
            Column::PreferredLanguage => "preferred_language",
            Column::MarketingOptIn => "marketing_opt_in",
            Column::TotalSpend => "total_spend",
            Column::OrderCount => "order_count",
            Column::Notes => "notes",
        }
    }

    /// 解析済み行における 0 始まりの位置です。
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// 整形済み CSV に書く正規ヘッダ行です。
pub const HEADER_ROW: [&str; 19] = [
    "CustomerID",
    "full_name",
    "email",
    "phone",
    "address_line",
    "city",
    "region",
    "postal_code",
    "country",
    "birth_date",
    "signup_date",
    "last_purchase_date",
    "status",
    "tier",
    "preferred_language",
    "marketing_opt_in",
    "total_spend",
    "order_count",
    "notes",
];
