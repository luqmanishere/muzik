use diesel::prelude::*;
use serde::Deserialize;

#[derive(Default, Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name=crate::schema::song)]
pub struct Song {
  pub id: i32,
  pub title: String,
  pub youtube_id: Option<String>,
  pub thumbnail_url: Option<String>,
  pub file_id: Option<i32>,
}

#[derive(Default, Associations, Insertable, Deserialize, PartialEq, Eq)]
#[diesel(belongs_to(File))]
#[diesel(table_name=crate::schema::song)]
pub struct NewSong {
  pub title: String,
  pub youtube_id: Option<String>,
  pub thumbnail_url: Option<String>,
  pub file_id: Option<i32>,
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq, Eq)]
#[diesel(table_name=crate::schema::artist)]
pub struct Artist {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name=crate::schema::artist)]
pub struct NewArtist {
  pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name=crate::schema::album)]
pub struct Album {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name=crate::schema::album)]
pub struct NewAlbum {
  pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name=crate::schema::genre)]
pub struct Genre {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name=crate::schema::genre)]
pub struct NewGenre {
  pub name: String,
}

#[derive(Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name=crate::schema::file)]
pub struct File {
  pub id: i32,
  pub relative_path: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name=crate::schema::file)]
pub struct NewFile {
  pub relative_path: String,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, Associations, Debug)]
#[diesel(table_name=crate::schema::songs_artists)]
#[diesel(belongs_to(Song))]
#[diesel(belongs_to(Artist))]
#[diesel(primary_key(song_id, artist_id))]
pub struct SongArtist {
  pub song_id: i32,
  pub artist_id: i32,
}

#[derive(Identifiable, Selectable, Insertable, Queryable, Associations, Debug)]
#[diesel(table_name=crate::schema::songs_albums)]
#[diesel(belongs_to(Song))]
#[diesel(belongs_to(Album))]
#[diesel(primary_key(song_id, album_id))]
pub struct SongAlbum {
  pub song_id: i32,
  pub album_id: i32,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, Associations, Debug)]
#[diesel(table_name=crate::schema::songs_genres)]
#[diesel(belongs_to(Song))]
#[diesel(belongs_to(Genre))]
#[diesel(primary_key(song_id, genre_id))]
pub struct SongGenre {
  pub song_id: i32,
  pub genre_id: i32,
}
