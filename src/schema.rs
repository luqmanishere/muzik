// @generated automatically by Diesel CLI.

diesel::table! {
    album (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    artist (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    file (id) {
        id -> Integer,
        relative_path -> Text,
    }
}

diesel::table! {
    genre (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    song (id) {
        id -> Integer,
        title -> Text,
        source -> Nullable<Text>,
        youtube_id -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        file_id -> Nullable<Integer>,
    }
}

diesel::table! {
    songs_albums (song_id, album_id) {
        song_id -> Integer,
        album_id -> Integer,
    }
}

diesel::table! {
    songs_artists (song_id, artist_id) {
        song_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    songs_genres (song_id, genre_id) {
        song_id -> Integer,
        genre_id -> Integer,
    }
}

diesel::joinable!(song -> file (file_id));
diesel::joinable!(songs_albums -> album (album_id));
diesel::joinable!(songs_albums -> song (song_id));
diesel::joinable!(songs_artists -> artist (artist_id));
diesel::joinable!(songs_artists -> song (song_id));
diesel::joinable!(songs_genres -> genre (genre_id));
diesel::joinable!(songs_genres -> song (song_id));

diesel::allow_tables_to_appear_in_same_query!(
  album,
  artist,
  file,
  genre,
  song,
  songs_albums,
  songs_artists,
  songs_genres,
);
