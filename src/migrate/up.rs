pub const MIGRATIONS: &[&str] = &[r#"
    CREATE TABLE IF NOT EXISTS statuses (
        id TEXT NOT NULL,
        series TEXT NOT NULL,
        status INTEGER NOT NULL,
        time INTEGER NOT NULL,
        PRIMARY KEY (`id`)
    );
    CREATE INDEX `series_idx` ON `series`;
"#];
