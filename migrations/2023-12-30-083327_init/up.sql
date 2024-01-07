-- Your SQL goes here
-- CreateTable
CREATE TABLE "song" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "title" TEXT NOT NULL,
    "source" TEXT,
    "youtube_id" TEXT,
    "thumbnail_url" TEXT,
    "file_id" INTEGER UNIQUE,
  FOREIGN KEY("file_id") REFERENCES file("id")
);

-- CreateTable
CREATE TABLE "artist" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

-- CreateTable
CREATE TABLE "album" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

-- CreateTable
CREATE TABLE "genre" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

-- CreateTable
CREATE TABLE "file" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "relative_path" TEXT NOT NULL UNIQUE
);

CREATE TABLE "songs_artists" (
    "song_id" INTEGER NOT NULL,
  "artist_id" INTEGER NOT NULL ,
  FOREIGN KEY("song_id") REFERENCES song("id"),
  FOREIGN KEY("artist_id") REFERENCES artist("id"),
    UNIQUE("song_id", "artist_id"),
    PRIMARY KEY("song_id", "artist_id")
);

CREATE TABLE "songs_albums" (
"song_id" INTEGER NOT NULL,
  "album_id" INTEGER NOT NULL,
  FOREIGN KEY("song_id") REFERENCES song("id"),
  FOREIGN KEY("album_id") REFERENCES album("id"),
    UNIQUE("song_id", "album_id"),
    PRIMARY KEY("song_id", "album_id")
);

CREATE TABLE "songs_genres" (
  "song_id" INTEGER NOT NULL,
  "genre_id" INTEGER NOT NULL,
  FOREIGN KEY("song_id") REFERENCES song("id"),
  FOREIGN KEY("genre_id") REFERENCES genre("id"),
    UNIQUE("song_id", "genre_id"),
    PRIMARY KEY("song_id", "genre_id")
)


