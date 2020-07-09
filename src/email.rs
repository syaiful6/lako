use std::env;
use std::path::Path;

use failure::Fail;
use lettre::file::FileTransport;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::SmtpClient;
use lettre::{SendableEmail, Transport};

use lettre_email::Email;

#[derive(Debug)]
pub struct SmtpConfig {
    pub username: String,
    pub password: String,
    pub server: String,
}

fn get_email_sender() -> (String, String) {
    let address = match env::var("MAIL_FROM_ADDRESS") {
        Ok(addr) => addr,
        _        => "test@localhost".to_string()
    };
    let name   = match env::var("MAIL_FROM_NAME") {
        Ok(name) => name,
        _        => "Lako".to_string()
    };
    (address, name)
}

pub fn init_smtp_config_vars() -> Option<SmtpConfig> {
    match (
        env::var("SMTP_USERNAME"),
        env::var("SMTP_PASSWORD"),
        env::var("SMTP_SERVER")
    ) {
        (Ok(username), Ok(password), Ok(server)) => Some(SmtpConfig {
            username: username,
            password: password,
            server: server,
        }),
        _ => None,
    }
}

pub fn send_user_confirm_email(email: &str, user_name: &str, token: &str) {
    let _ = try_send_user_confirm_email(email, user_name, token);
}

pub fn try_send_user_confirm_email(email: &str, user_name: &str, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Please confirm your email address";
    let body = format!(
        "Hello {}! Welcome to Lako. Please click the
link below to verify your email address. Thank you!\n
https://lako.io/confirm/{}",
        user_name, token
    );

    send_email(email, subject, &body)
}

fn build_email(
    recipient: &str,
    subject: &str,
    body: &str
) -> Result<SendableEmail, Box<dyn std::error::Error>> {
    let email = Email::builder()
        .to(recipient)
        .from(get_email_sender())
        .subject(subject)
        .body(body)
        .build()
        .map_err(|e| e.compat())?;

    Ok(email.into())
}

fn send_email(recipient: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_config = init_smtp_config_vars();
    let email = build_email(recipient, subject, body)?;

    match smtp_config {
        Some(smtp_config) => {
            let mut transport = SmtpClient::new_simple(&smtp_config.server)?
                .credentials(Credentials::new(
                    smtp_config.username,
                    smtp_config.password,
                ))
                .smtp_utf8(true)
                .authentication_mechanism(Mechanism::Plain)
                .transport();
            
            transport.send(email)?;
        }
        None => {
            let mut sender = FileTransport::new(Path::new("/tmp"));
            sender.send(email)?;
        }
    }

    Ok(())
}