use std::path;
use std::fs::File;
use std::sync::{Arc, Mutex, atomic};
use rusqlite;

use server::ServerError;

#[derive(Clone)]
pub struct DatabaseRef {
    location: path::PathBuf,
    db_ref: Arc<Mutex<rusqlite::Connection>>,
    current_play_num: Arc<atomic::AtomicUsize>,
    current_game_num: Arc<usize>,
    in_transaction: Arc<atomic::AtomicBool>,
}

impl DatabaseRef {
    pub fn new_database(loc: &path::Path) -> Result<DatabaseRef, ServerError> {
        if !loc.exists() {
            let _ = try!(File::create(loc));
        }

        let mut connection = try!(rusqlite::Connection::open(loc.to_owned()));

        connection.execute("CREATE TABLE IF NOT EXISTS games (id INTEGER PRIMARY KEY ASC, date TEXT)", &[])?;
        connection.execute("CREATE TABLE IF NOT EXISTS plays (id INTEGER PRIMARY KEY ASC, game_id INTEGER, FORIEGN KEY(game_id) REFERENCES games(id))", &[])?;
        connection.execute("CREATE TABLE IF NOT EXISTS clips (id INTEGER PRIMARY KEY ASC, uuid TEXT, play_id INTEGER, FORIEGN KEY(play_id) REFERENCES plays(id))", &[])?;

        connection.execute("INSERT INTO games (date) VALUES (date('now'))", &[])?;
        let current_game_num: u32 = connection.query_row("SELECT id FROM games WHERE date = date('now') ORDER BY id", &[], |ref row| row.get(0))?;

        
        Ok(
            DatabaseRef {
                location: loc.to_owned(),
                db_ref: Arc::new(Mutex::new(connection)),
                current_play_num: Arc::new(atomic::AtomicUsize::new(1)),
                current_game_num: Arc::new(current_game_num as usize),
                in_transaction: Arc::new(atomic::AtomicBool::new(false)),
            }
        )
    }

    pub fn start_play(&mut self) -> rusqlite::Result<()> {
        if self.in_transaction.load(atomic::Ordering::SeqCst) {
            panic!("Attempted to start a play when one was already started");
        }

        let lock = self.db_ref.lock().expect("mutex is poisoned");
        lock.execute("INSERT INTO plays (game_id) VALUES (?)", &[&(*self.current_game_num as u32)])?;
        self.in_transaction.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    pub fn insert_clip(&mut self, uuid: &str) -> rusqlite::Result<()> {
        if self.in_transaction.load(atomic::Ordering::SeqCst) {
            panic!("Attempted to add a clip when there was no play started");
        }

        let lock = self.db_ref.lock().expect("mutex is poisoned");
        let actual_play_id = self.current_play_num.load(atomic::Ordering::SeqCst);
        lock.execute("INSERT INTO clips (uuid, game_id) VALUES (?, ?)", &[&uuid, &(actual_play_id as u32)])?;

        Ok(())
    }

    pub fn end_play(&mut self) -> bool {
        self.current_play_num.fetch_add(1, atomic::Ordering::SeqCst);
        self.in_transaction.swap(false, atomic::Ordering::SeqCst)
    }

    pub fn currently_in_play(&self) -> bool {
        self.in_transaction.load(atomic::Ordering::SeqCst)
    }

}