use std::env;

use lettre::smtp::authentication::Credentials;
use lettre::smtp::client::net::{ClientTlsParameters, DEFAULT_TLS_PROTOCOLS};
use lettre::smtp::{ClientSecurity, SmtpClient, SmtpTransport, SUBMISSION_PORT};
use lettre::{SendableEmail, Transport};
use native_tls::TlsConnector;

use crate::error::AppError;
use lettre_email::Email;

#[derive(Debug)]
pub struct SmtpConfig {
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u16,
}

fn get_email_sender() -> (String, String) {
    let address = match env::var("MAIL_FROM_ADDRESS") {
        Ok(addr) => addr,
        _ => "test@localhost".to_string(),
    };
    let name = match env::var("MAIL_FROM_NAME") {
        Ok(name) => name,
        _ => "Lako".to_string(),
    };
    (address, name)
}

pub fn init_smtp_config_vars() -> Option<SmtpConfig> {
    match (
        env::var("SMTP_USERNAME"),
        env::var("SMTP_PASSWORD"),
        env::var("SMTP_SERVER"),
        env::var("SMTP_PORT"),
    ) {
        (Ok(username), Ok(password), Ok(server), Ok(port)) => Some(SmtpConfig {
            username: username,
            password: password,
            server: server,
            port: port.parse::<u16>().unwrap_or(SUBMISSION_PORT),
        }),
        (Ok(username), Ok(password), Ok(server), _) => Some(SmtpConfig {
            username: username,
            password: password,
            server: server,
            port: SUBMISSION_PORT,
        }),
        _ => None,
    }
}

pub fn send_user_confirm_email(email: &str, user_name: &str, token: &str) {
    let _ = try_send_user_confirm_email(email, user_name, token);
}

pub fn try_send_user_confirm_email(
    email: &str,
    user_name: &str,
    token: &str,
) -> Result<(), AppError> {
    let subject = "Please confirm your email address";
    let body = format!(
        "Hello {}! Welcome to Lako. Please click the
link below to verify your email address. Thank you!\n
https://lako.io/confirm/{}",
        user_name, token
    );

    send_email(email, subject, &body)
}

fn build_email(recipient: &str, subject: &str, body: &str) -> Result<SendableEmail, AppError> {
    let email = Email::builder()
        .to(recipient)
        .from(get_email_sender())
        .subject(subject)
        .body(body)
        .build()?;

    Ok(email.into())
}

fn smtp_use_ssl() -> bool {
    match env::var("SMPT_USE_SSL") {
        Ok(s) => s == "yes" || s == "1",
        _ => false,
    }
}

fn create_smtp_transport() -> Result<SmtpTransport, AppError> {
    let smtp_config = init_smtp_config_vars();
    match smtp_config {
        Some(smtp_config) => {
            let security = {
                if smtp_use_ssl() {
                    let tls_connector = TlsConnector::builder()
                        .min_protocol_version(Some(DEFAULT_TLS_PROTOCOLS[0]))
                        .build()?;

                    let domain = smtp_config.server.to_owned();
                    ClientSecurity::Required(ClientTlsParameters::new(domain, tls_connector))
                } else {
                    ClientSecurity::None
                }
            };

            let transport =
                SmtpClient::new((smtp_config.server.as_str(), smtp_config.port), security)?
                    .credentials(Credentials::new(smtp_config.username, smtp_config.password))
                    .smtp_utf8(true)
                    .transport();

            Ok(transport)
        }
        None => Err(AppError::InvalidConfig),
    }
}

fn send_email(recipient: &str, subject: &str, body: &str) -> Result<(), AppError> {
    let email = build_email(recipient, subject, body)?;

    let mut transport = create_smtp_transport()?;
    transport.send(email)?;

    Ok(())
}
