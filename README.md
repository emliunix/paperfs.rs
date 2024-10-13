# paperfs.rs

1. paper because I use it with zotero
2. fs because it's a webdav server to work with zotero webdav sync
3. rs because it's written in Rust

And I use it to connect to my personal onedrive.

Still working towards MVP.

## TODO

* [ ] rename odrive.rs to msauth.rs,
* [ ] on access token refresh, only reconstruct opendal operator. Current approach may corrupt the WebDAV fslock semantic.
* [ ] add workflow and automated deployment
* [ ] UI for login, though curl works