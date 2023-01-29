pub const MIGRATIONS: &'static [&'static str] = &[r#"
    CREATE TABLE IF NOT EXISTS checks (
        id TEXT NOT NULL,
        url TEXT NOT NULL,
        frequency INTEGER CHECK(frequency IN (5, 15, 30, 90)),
        PRIMARY KEY (`id`)
    );
"#];
