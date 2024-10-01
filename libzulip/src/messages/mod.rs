// each module represents one API call. this allows for lots of strong-typed
// happiness without the massive overload of random types everywhere
pub mod delete_message;
pub mod download_file; // note: this isn't an api call. it's here for sanity
pub mod edit_message;
pub mod send;
pub mod upload_file;
