# picvu

My [Not-Invented-Here](https://en.wikipedia.org/wiki/Not_invented_here) attempt at a photo organisation app.

I've grown pretty sick of Google Photos. It's hard to export data - yes I've
looked at [Takeout](https://takeout.google.com/), but a lot of the data is hidden.
I've tried using the [Google Photos API](https://developers.google.com/photos) -
but again lots of the data is hidden, and you can't modify meta-data for images
that were not uploaded to your application.

I want my own application that organises photos (and other media - including
recepies), with clear import and export functions.

So what is it?

It is:
1. A personal project that lets me sort out my personal projects.
2. A really good, solid project for me to flesh out my
   [Rust](https://www.rust-lang.org/) skills.
3. Software that I actually use.

It isn't:
1. A website/application that can really be hosted on the public internet.
   1. It doesn't have any access control or concept of user accounts.
   2. It explicitly interracts with the local file system of the computer
      it's running on.
2. An view into the Google Photos data - modification of Google Photos
   data is really expliclty not supported by the API.

# Building
1. Install [Rust](https://www.rust-lang.org/)
2. `cargo run --release`
3. NOTE - you will see *massive* speed improvements in release mode -
   particularly in relation to image processing (e.g. generating
   thumbnails).
4. Visit http://localhost:8080/
5. See that data is stored in a file `picvu.db` in the current directory.
   This is a [SQLite](https://www.sqlite.org/) database. You can always
   extract data using standard SQLite tools.
6. TODO: instructions to create appropriate Google credentials.
7. Visit the "Setup" page, and specify:
   1. An API key to access the Google TimeZone and Geocoding APIs.
   2. A Client ID and Secret for a Google app that has access to
      the Google Photos API with the https://www.googleapis.com/auth/photoslibrary
      authorization scope.
8. Visit [Google Takeout](https://takeout.google.com) and extract all of your
   [Google Photos](https://photos.google.com) photos, in a set of .tgz files.
9. Download all of these files and place them into a folder.
10. Visit the "Import" page, and paste in the path to this folder.
11. Import and wait.......
12. Photos are tagged with "Unsorted" and some geogrphic information
    (from the Goolgle Geocoding API) by default.
13. Import other folders of photos.
14. Start tagging and sorting the photos.

# Third Party Software
Third party software used by this project includes:
1. [Rust](https://rust-lang.org/), including lots of
   crates from [Cargo](https://crates.io/).
2. [SQLite](https://www.sqlite.org/), via the
   [Diesel](https://crates.io/crates/diesel) and
   [Rusqlite](https://crates.io/crates/rusqlite) packages, with
   the Ruqlite "bundled" feature.
3. [Bootstrap Icons](https://icons.getbootstrap.com/) - included as files
   in src/picvu/assests/bootstrap-icons.css and sub-folder fonts.
   Downloaded from https://github.com/twbs/icons/releases/download/v1.2.2/bootstrap-icons-1.2.2.zip
   on 2021-01-06.
4. ~~[Feather Icons](https://feathericons.com/) - included as file
   src/picvu/assets/feather-sprite.svg (Obtained from
   https://unpkg.com/feather-icons/dist/feather-sprite.svg on 2020-07-04).~~