use rocket::{
    response::{self, NamedFile, Responder, Response},
    Request,
};

pub struct FileDownload(pub NamedFile, pub String);

impl Responder<'_> for FileDownload {
    fn respond_to(self, req: &Request<'_>) -> response::Result<'static> {
        Response::build()
            .merge(self.0.respond_to(req)?)
            .raw_header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.1),
            )
            .ok()
    }
}
