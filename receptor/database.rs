use rusqlite::Connection;


pub fn initialize_database() {
    let conn = Connection::open("database.sqlite").expect("Can't open database connection");

    conn.execute("CREATE TABLE IF NOT EXISTS raw_status (
        sender TEXT NOT NULL,
        message TEXT NOT NULL,
        plugin_name TEXT NOT NULL,
        ts_sent DATETIME NOT NULL,
        ts_received DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
    )", &[]).expect("Can't create raw status table");

    conn.execute("CREATE INDEX IF NOT EXISTS ts_sent_ind ON raw_status(ts_sent)", &[]).expect("Can't index tables");
    conn.execute("CREATE INDEX IF NOT EXISTS ts_received_ind ON raw_status(ts_received)", &[]).expect("Can't index tables");

    conn.execute("CREATE TABLE IF NOT EXISTS processed_status (
    sender TEXT NOT NULL,
    message TEXT NOT NULL,
    plugin_name TEXT NOT NULL,
    ts DATETIME DEFAULT CURRENT_TIMESTAMP
    )", &[]).expect("Can't create processed status table");

    conn.execute("CREATE INDEX IF NOT EXISTS ts_sent_ind ON processed_status(ts_sent)", &[]).expect("Can't index tables");
    conn.execute("CREATE INDEX IF NOT EXISTS ts_received_ind ON processed_status(ts_received)", &[]).expect("Can't index tables");

    conn.close().expect("Can't close connection to sqlite");
}

pub fn get_connection() -> Connection {
    let conn = Connection::open("database.sqlite").expect("Can't open database connection");
    return conn;
}
