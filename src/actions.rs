use diesel::prelude::*;
use diesel::dsl;

use crate::models;

pub fn find_person_by_id(
    person_id: i32,
    conn: &SqliteConnection
) -> Result<Option<models::Person>, diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    let person = persons
        .filter(id.eq(person_id))
        .first::<models::Person>(conn)
        .optional()?;

    Ok(person)
}

pub fn get_all_persons(
    conn: &SqliteConnection
) -> Result<Vec<models::Person>, diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    persons.load::<models::Person>(conn)
}

fn next_id(conn: &SqliteConnection) -> Result<i32, diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    let max_id: Option<i32> = persons.select(diesel::dsl::max(id)).first(conn)?;

    let next_id = if let Some(max_id) = max_id {
        max_id + 1
    } else {
        1
    };
    Ok(next_id)
}

pub fn insert_new_person(
    person_name: &str,
    conn: &SqliteConnection,
) -> Result<models::Person, diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    //let next_id = persons.select(diesel::dsl::max(id)).first(conn).optional()?.map_or_else(|| 1, |i: i32| i + 1);
    let next_id = next_id(conn)?;
    let new_person = models::Person {
        id: next_id,
        name: person_name.to_owned(),
    };

    diesel::insert_into(persons).values(&new_person).execute(conn)?;

    Ok(new_person)
}

pub fn update_person(
    person_id: i32,
    new_name: &str,
    conn: &SqliteConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    diesel::update(persons.filter(id.eq(person_id)))
        .set(name.eq(new_name))
        .execute(conn)?;

    Ok(())
}

pub fn delete_person(
    person_id: i32,
    conn: &SqliteConnection,
    ) -> Result<(), diesel::result::Error> {
    use crate::schema::persons::dsl::*;

    diesel::delete(persons.filter(id.eq(person_id)))
        .execute(conn)?;
    Ok(())
}
