use serenity::model::channel::Message;

fn is_mentioning_everyone(content: &String) -> bool {
    return content.contains("@everyone") || content.contains("@here");
}

pub fn is_spam(message: &Message) -> bool {
    // the mention_everyone field is false if the user does not have the permissions for it
    // this means you have to do a manual search

    // TODO: check guild roles if everyone can ping everyone
    // true if the user has permissions for it, in which case we don't care
    if message.mention_everyone {
        return false;
    }

    // TODO: cover very frequent spam, n+ messages within x seconds

    if is_mentioning_everyone(&message.content) {
        return true;
    }

    return false;
}

#[cfg(test)]
mod tests {
    use crate::message_handler::is_mentioning_everyone;

    #[test]
    fn test_is_mentioning_everyone() {
        // normal text
        assert!(is_mentioning_everyone(&"@everyone".to_string()));
        assert!(is_mentioning_everyone(&"@here".to_string()));
        // TODO: inline code
        assert!(is_mentioning_everyone(&"`@everyone`".to_string()));
        assert!(is_mentioning_everyone(&"`@here`".to_string()));
        // TODO: code block
        assert!(is_mentioning_everyone(&"```txt\n@everyone\n```".to_string()));
        assert!(is_mentioning_everyone(&"```txt\n@here\n```".to_string()));
        // inline quote
        assert!(is_mentioning_everyone(&"> @everyone".to_string()));
        assert!(is_mentioning_everyone(&"> @here".to_string()));
        // block quote
        assert!(is_mentioning_everyone(&">>> @everyone".to_string()));
        assert!(is_mentioning_everyone(&">>> @here".to_string()));
    }
}
