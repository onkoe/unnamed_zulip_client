// each module represents one API call. this allows for lots of strong-typed
// happiness without the massive overload of random types everywhere

// message modules
pub mod delete_message;
pub mod edit_message;
pub mod fetch_single_message;
pub mod send_message;

// media modules
pub mod download_file;
pub mod emoji_reaction; // contains both add and remove calls
pub mod upload_file; // note: this isn't an api call. it's here for sanity
