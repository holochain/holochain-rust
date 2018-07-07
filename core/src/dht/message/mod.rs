enum Type {
    Response {
        Ok = "OK_RESPONSE",
        Err = "ERROR_RESPONSE",
    },
    Request {
        Put = "PUT_REQUEST",
        Del = "DEL_REQUEST",
        Mod = "MOD_REQUEST",
        Get = "GET_REQUEST",
        Link = "LINK_REQUEST",
        GetLink = "GETLINK_REQUEST",
        DeleteLink = "DELETELINK_REQUEST",
        Gossip = "GOSSIP_REQUEST",
    },
    Validate {
        Put: "VALIDATE_PUT_REQUEST"
    }
}

pub struct Message {
    message_type: Type,
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn responses() {
        assert_eq!(Message::Response::Ok, "OK_RESPONSE");
        assert_eq!(Message::Response::Err, "OK_RESPONSE");
    }

}
