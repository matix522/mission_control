// @generated automatically by Diesel CLI.

diesel::table! {
    missions (mission_id) {
        mission_id -> Int4,
        mission_name -> Varchar,
        location -> Varchar,
        tags -> Array<Nullable<Text>>,
    }
}
