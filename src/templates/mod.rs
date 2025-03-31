use askama::Template; // bring trait in scope

#[derive(Template)] // this will generate the code...
#[template(path = "/Users/harshranjan/snippetbox-rs/templates/home.html")] // using the template in this path, relative
// to the `templates` dir in the crate root
// struct HelloTemplate<'a> {
pub struct HomeTemplate {
    // the name of the struct can be anything
    // name: &'a str, // the field name should match the variable name
    // in your template
}
