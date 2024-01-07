use color_eyre::eyre::{eyre, Context, Result};
use diesel::{prelude::*, Connection, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use tracing::debug;

use crate::{
  config::Config,
  models::{
    Album, Artist, Genre, NewAlbum, NewArtist, NewFile, NewGenre, NewSong, Song, SongAlbum, SongArtist, SongGenre,
  },
  schema::{album, artist, genre, song, songs_artists},
};
use std::path::{Path, PathBuf};

pub struct Database {
  connection: SqliteConnection,
  config: Config,
}

impl Database {
  /// Initialize a new instance of Database
  ///
  /// # Arguments
  ///
  /// * config: the `Config` used by the application
  ///
  /// # Returns
  ///
  /// * an instance of `Database` wrapped in a `Result`
  pub async fn new(config: Config) -> Result<Self> {
    // use local database if using debug builds
    // database should be determined by config otherwise

    #[cfg(not(debug_assertions))]
    let connection = {
      let url = format!("file:{}", config.config._data_dir.join("database.db").display().to_string());
      SqliteConnection::establish(&url).wrap_err("establish sqlite connection")?
    };

    #[cfg(debug_assertions)]
    let connection = SqliteConnection::establish("file:./dev.db").wrap_err("establish sqlite connection")?;

    // TODO: run migrations if available

    Ok(Self { connection, config })
  }

  /// Insert a `NewSong` into the database
  ///
  /// # Arguments
  ///
  /// * `new_song` - the song to be inserted
  ///
  /// # Returns
  ///
  /// * the id of the new entry wrapped in a `Result`
  pub fn insert_song(&mut self, new_song: NewSong) -> Result<i32> {
    use crate::schema::song::dsl::*;
    let res = diesel::insert_into(song).values(&new_song).returning(id).get_result::<i32>(&mut self.connection)?;
    Ok(res)
  }

  /// Insert an `Artist` into the database. If there is an existing entry with the same name, will
  /// return the id of the existing entry
  ///
  /// # Arguments
  ///
  /// * `new_artist` - struct containing the name of the artist
  ///
  /// # Returns
  ///
  /// * the id of the inserted `Artist` wrapped in a Result
  pub fn insert_artist(&mut self, new_artist: NewArtist) -> Result<i32> {
    use crate::schema::artist::dsl::*;

    let artist_id: i32 = match crate::schema::artist::table
      .filter(name.eq(&new_artist.name))
      .select(id)
      .get_result(&mut self.connection)
    {
      Ok(artist_id) => artist_id,
      Err(e) => match e {
        diesel::result::Error::NotFound => {
          diesel::insert_into(artist).values(&new_artist).returning(id).get_result(&mut self.connection)?
        },
        _ => {
          return Err(e.into());
        },
      },
    };
    Ok(artist_id)
  }

  /// Insert an `Album` into the database. If there is an existing entry with the same name, will
  /// return the id of the existing entry
  ///
  /// # Arguments
  ///
  /// * `new_album` - struct containing the name of the album
  ///
  /// # Returns
  ///
  /// * the id of the inserted `Album` wrapped in a Result
  pub fn insert_album(&mut self, new_album: NewAlbum) -> Result<i32> {
    use crate::schema::album::dsl::*;

    let album_id: i32 =
      match crate::schema::album::table.filter(name.eq(&new_album.name)).select(id).get_result(&mut self.connection) {
        Ok(album_id) => album_id,
        Err(e) => match e {
          diesel::result::Error::NotFound => {
            diesel::insert_into(album).values(&new_album).returning(id).get_result(&mut self.connection)?
          },
          _ => {
            return Err(e.into());
          },
        },
      };
    Ok(album_id)
  }

  /// Insert a `Genre` into the database. If there is an existing entry with the same name, will
  /// return the id of the existing entry
  ///
  /// # Arguments
  ///
  /// * `new_genre` - struct containing the name of the genre
  ///
  /// # Returns
  ///
  /// * the id of the inserted `genre` wrapped in a Result
  pub fn insert_genre(&mut self, new_genre: NewGenre) -> Result<i32> {
    use crate::schema::genre::dsl::*;

    let genre_id: i32 =
      match crate::schema::genre::table.filter(name.eq(&new_genre.name)).select(id).get_result(&mut self.connection) {
        Ok(genre_id) => genre_id,
        Err(e) => match e {
          diesel::result::Error::NotFound => {
            diesel::insert_into(genre).values(&new_genre).returning(id).get_result(&mut self.connection)?
          },
          _ => {
            return Err(e.into());
          },
        },
      };
    Ok(genre_id)
  }

  pub fn insert_file(&mut self, new_file: NewFile) -> Result<i32> {
    use crate::schema::file::dsl::*;
    let file_id: i32 = match crate::schema::file::table
      .filter(relative_path.eq(&new_file.relative_path))
      .select(id)
      .get_result(&mut self.connection)
    {
      Ok(file_id) => file_id,
      Err(e) => match e {
        diesel::result::Error::NotFound => {
          diesel::insert_into(file).values(&new_file).returning(id).get_result(&mut self.connection)?
        },
        _ => {
          return Err(e.into());
        },
      },
    };
    Ok(file_id)
  }

  pub fn insert_song_artist(&mut self, new_song_artist: SongArtist) -> Result<()> {
    use crate::schema::songs_artists::dsl::*;

    diesel::insert_into(songs_artists).values(new_song_artist).execute(&mut self.connection)?;
    Ok(())
  }

  pub fn insert_song_album(&mut self, new_song_album: SongAlbum) -> Result<()> {
    use crate::schema::songs_albums::dsl::*;

    diesel::insert_into(songs_albums).values(new_song_album).execute(&mut self.connection)?;
    Ok(())
  }

  pub fn insert_song_genre(&mut self, new_song_genre: SongGenre) -> Result<()> {
    use crate::schema::songs_genres::dsl::*;

    diesel::insert_into(songs_genres).values(new_song_genre).execute(&mut self.connection)?;
    Ok(())
  }

  pub fn get_song_from_id(&mut self, song_id: i32) -> Result<Song> {
    let song = crate::schema::song::table.find(song_id).select(Song::as_select()).first(&mut self.connection)?;
    Ok(song)
  }

  pub fn get_all_songs(&mut self) -> Result<Vec<Song>> {
    let all_songs: Vec<Song> = song::table.select(Song::as_select()).load(&mut self.connection)?;

    debug!("{:?}", &all_songs);

    /*
    let artists = SongArtist::belonging_to(&all_songs)
      .inner_join(artist::table)
      .select((SongArtist::as_select(), Artist::as_select()))
      .load(&mut self.connection)?;
    debug!("{:?}", &artists);

    let artists_per_song: Vec<(Song, Vec<Artist>)> = artists
      .grouped_by(&all_songs)
      .into_iter()
      .zip(all_songs)
      .zip(albums_per_song).zip()
      .map(|(artist, song)| (song, artist.into_iter().map(|(_, artist)| artist).collect()))
      .collect();
    */

    Ok(all_songs)
  }

  pub fn get_all_artists_for_song(&mut self, song: Song) -> Result<Vec<Artist>> {
    let artists: Vec<Artist> = SongArtist::belonging_to(&song)
      .inner_join(artist::table)
      .select(artist::all_columns)
      .load(&mut self.connection)?;
    Ok(artists)
  }
}

#[cfg(test)]
mod tests {
  use color_eyre::eyre::{Context, Result};
  use diesel::prelude::*;
  use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
  use pretty_assertions::assert_eq;

  use crate::{
    config::Config,
    models::{NewAlbum, NewArtist, NewGenre, NewSong, Song, SongArtist},
  };

  use super::*;

  // embed migrations into tests
  pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

  /// Spawns an instance of `Database` with a new instance of in memory sqlite database for tests
  fn setup_database() -> Result<Database> {
    let mut connection = SqliteConnection::establish(":memory:").wrap_err("establish sqlite connection")?;
    connection.run_pending_migrations(MIGRATIONS).expect("migration successful");
    let database = Database { connection, config: Config::default() };
    Ok(database)
  }

  #[test]
  fn test_database_get_all_songs() -> Result<()> {
    let mut database = setup_database()?;
    let insert1 = database.insert_song(NewSong { title: "Stellar Stellar".to_string(), ..Default::default() })?;
    let insert2 = database.insert_song(NewSong { title: "Crossing Field".to_string(), ..Default::default() })?;
    let insert3 = database.insert_song(NewSong { title: "Loli God Requiem".to_string(), ..Default::default() })?;

    let songs = database.get_all_songs()?;
    let songs_check = vec![
      Song { id: 1, title: "Stellar Stellar".to_string(), ..Default::default() },
      Song { id: 2, title: "Crossing Field".to_string(), ..Default::default() },
      Song { id: 3, title: "Loli God Requiem".to_string(), ..Default::default() },
    ];

    assert_eq!(songs, songs_check);
    Ok(())
  }

  #[test]
  fn test_database_get_all_artists_for_song() -> Result<()> {
    let mut database = setup_database()?;

    let new_song = NewSong { title: "Stellar Stellar".to_string(), ..Default::default() };
    let song_id = database.insert_song(new_song)?;
    let artist1_id = database.insert_artist(NewArtist { name: "Hoshimachi Suisei".to_string() })?;
    let artist2_id = database.insert_artist(NewArtist { name: "Comet-chan".to_string() })?;
    database.insert_song_artist(SongArtist { song_id, artist_id: artist1_id })?;
    database.insert_song_artist(SongArtist { song_id, artist_id: artist2_id })?;

    let song = database.get_song_from_id(song_id)?;
    let artists = database.get_all_artists_for_song(song)?;
    assert_eq!(
      artists,
      vec![Artist { id: 1, name: "Hoshimachi Suisei".to_string() }, Artist { name: "Comet-chan".to_string(), id: 2 }]
    );
    Ok(())
  }

  #[test]
  fn test_database_artist_insert_conflict() -> Result<()> {
    let mut database = setup_database()?;
    let insert1 = database.insert_artist(NewArtist { name: "Suisei".to_string() })?;
    let insert2 = database.insert_artist(NewArtist { name: "Suisei".to_string() })?;
    let insert3 = database.insert_artist(NewArtist { name: "LiSA".to_string() })?;
    assert_eq!(insert1, insert2);
    assert_eq!(insert3, 2);
    Ok(())
  }

  #[test]
  fn test_database_album_insert_conflict() -> Result<()> {
    let mut database = setup_database()?;
    let insert1 = database.insert_album(NewAlbum { name: "Still Still Stellar".to_string() })?;
    let insert2 = database.insert_album(NewAlbum { name: "Still Still Stellar".to_string() })?;
    let insert3 = database.insert_album(NewAlbum { name: "Sword Art Online OSTs".to_string() })?;
    assert_eq!(insert1, insert2);
    assert_eq!(insert3, 2);
    Ok(())
  }

  #[test]
  fn test_database_genre_insert_conflict() -> Result<()> {
    let mut database = setup_database()?;
    let insert1 = database.insert_genre(NewGenre { name: "Japanese Pop".to_string() })?;
    let insert2 = database.insert_genre(NewGenre { name: "Japanese Pop".to_string() })?;
    let insert3 = database.insert_genre(NewGenre { name: "Japanese Rock".to_string() })?;
    assert_eq!(insert1, insert2);
    assert_eq!(insert3, 2);
    Ok(())
  }

  #[test]
  fn test_database_song_artist_insert_conflict() -> Result<()> {
    let mut database = setup_database()?;
    let song_id = database.insert_song(NewSong { title: "Stellar Stellar".to_string(), ..Default::default() })?;
    let artist_id = database.insert_artist(NewArtist { name: "Hoshimachi Suisei".to_string() })?;

    database.insert_song_artist(SongArtist { song_id, artist_id })?;
    // this should return an error
    assert!(database.insert_song_artist(SongArtist { song_id, artist_id }).is_err());

    Ok(())
  }
}
