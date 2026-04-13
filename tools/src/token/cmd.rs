/*
 * Copyright (c) Huawei Technologies Co., Ltd. 2026. All rights reserved.
 * Global Trust Authority is licensed under the Mulan PSL v2.
 * You can use this software according to the terms and conditions of the Mulan PSL v2.
 * You may obtain a copy of Mulan PSL v2 at:
 *     http://license.coscl.org.cn/MulanPSL2
 * THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR
 * PURPOSE.
 * See the Mulan PSL v2 for more details.
 */

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{ArgAction, Args, Subcommand, ValueEnum};
use openssl::ecdsa::EcdsaSig;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{Id, PKey, Private};
use openssl::rsa::Padding;
use openssl::sign::{RsaPssSaltlen, Signer};
use serde_json::{Map, Value};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

use crate::common::formatter::Formatter as OutputFormatter;
use crate::common::validate::validate_file_path;
use crate::config::GlobalOptions;
use crate::error::{CliError, Result};
use crate::token::Token;

const DEFAULT_ISSUER: &str = "rbs-cli";
const DEFAULT_SUBJECT: &str = "administrator";
const DEFAULT_AUDIENCE: &str = "globaltrustauthority-rbs";
const DEFAULT_ROLE: &str = "admin";
const DEFAULT_EXP_AFTER_SECONDS: u64 = 3600;
const PRIVATE_KEY_ENV: &str = "RBS_ADMIN_PRIVATE_KEY";
const PRIVATE_KEY_FILE_ENV: &str = "RBS_ADMIN_PRIVATE_KEY_FILE";
const SUPPORTED_PRIVATE_KEYS: &str =
    "supported private keys: RSA for RS*/PS*, P-256 for ES256, P-384 for ES384, P-521 for ES512, SM2 for SM2, Ed25519/Ed448 for EdDSA";

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum TokenAlg {
    #[value(name = "RS256")]
    Rs256,
    #[value(name = "RS384")]
    Rs384,
    #[value(name = "RS512")]
    Rs512,
    #[value(name = "PS256")]
    Ps256,
    #[value(name = "PS384")]
    Ps384,
    #[value(name = "PS512")]
    Ps512,
    #[value(name = "SM2")]
    Sm2,
    #[value(name = "ES256")]
    Es256,
    #[value(name = "ES384")]
    Es384,
    #[value(name = "ES512")]
    Es512,
    #[default]
    #[value(name = "EdDSA")]
    Eddsa,
}

impl Display for TokenAlg {
    /// Formats the JWT alg value as the standard header string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rs256 => write!(f, "RS256"),
            Self::Rs384 => write!(f, "RS384"),
            Self::Rs512 => write!(f, "RS512"),
            Self::Ps256 => write!(f, "PS256"),
            Self::Ps384 => write!(f, "PS384"),
            Self::Ps512 => write!(f, "PS512"),
            Self::Sm2 => write!(f, "SM2"),
            Self::Es256 => write!(f, "ES256"),
            Self::Es384 => write!(f, "ES384"),
            Self::Es512 => write!(f, "ES512"),
            Self::Eddsa => write!(f, "EdDSA"),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct TokenCli {
    #[command(subcommand)]
    pub command: TokenCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TokenCommand {
    Generate(GenerateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GenerateArgs {
    #[arg(long, value_parser = validate_file_path)]
    pub private_key_file: Option<String>,

    #[arg(long, num_args = 0..=1, value_name = "@PATH")]
    pub private_key_passphrase: Option<Option<String>>,

    #[arg(long, default_value = DEFAULT_ISSUER)]
    pub iss: String,

    #[arg(long, default_value = DEFAULT_SUBJECT)]
    pub sub: String,

    #[arg(long, action = ArgAction::Append, default_value = DEFAULT_AUDIENCE)]
    pub aud: Vec<String>,

    #[arg(long, default_value = DEFAULT_ROLE)]
    pub role: String,

    #[arg(long)]
    pub exp: Option<u64>,

    #[arg(long)]
    pub nbf: Option<u64>,

    #[arg(long)]
    pub iat: Option<u64>,

    #[arg(long)]
    pub jti: Option<String>,

    #[arg(long, value_enum)]
    pub alg: Option<TokenAlg>,

    #[arg(long)]
    pub kid: Option<String>,

    #[arg(long)]
    pub claims: Option<String>,
}

impl Default for GenerateArgs {
    /// Builds the same defaults clap applies for token generation arguments.
    fn default() -> Self {
        Self {
            iss: DEFAULT_ISSUER.to_string(),
            sub: DEFAULT_SUBJECT.to_string(),
            aud: vec![DEFAULT_AUDIENCE.to_string()],
            role: DEFAULT_ROLE.to_string(),
            private_key_file: None,
            private_key_passphrase: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
            alg: None,
            kid: None,
            claims: None,
        }
    }
}

/// Dispatches token subcommands to the corresponding implementation.
pub fn run(cli: &TokenCli, _global: &GlobalOptions) -> Result<Box<dyn OutputFormatter>> {
    match &cli.command {
        TokenCommand::Generate(args) => run_generate(args),
    }
}

/// Generates a token and returns it as formatter output.
fn run_generate(args: &GenerateArgs) -> Result<Box<dyn OutputFormatter>> {
    Ok(Box::new(generate_token(args)?))
}

/// Builds and signs a JWT using the provided generation arguments.
fn generate_token(args: &GenerateArgs) -> Result<Token> {
    let private_key = load_private_key(args)?;
    let alg = match &args.alg {
        Some(alg) => {
            ensure_key_matches_alg(&private_key, alg)?;
            alg.clone()
        },
        None => infer_default_alg(&private_key)?,
    };

    // JWT signatures are calculated over the ASCII form `base64url(header).base64url(payload)`.
    let signing_input = build_signing_input(args, &alg)?;
    let signature = sign(signing_input.as_bytes(), &private_key, &alg)?;
    Ok(Token { token: format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(signature)) })
}

/// Builds the base64url-encoded JWT header and payload signing input.
fn build_signing_input(args: &GenerateArgs, alg: &TokenAlg) -> Result<String> {
    let mut header = Map::new();
    header.insert("typ".to_string(), Value::String("JWT".to_string()));
    header.insert("alg".to_string(), Value::String(alg.to_string()));
    if let Some(kid) = &args.kid {
        header.insert("kid".to_string(), Value::String(kid.clone()));
    }

    let mut payload = Map::new();
    payload.insert("iss".to_string(), Value::String(args.iss.clone()));
    payload.insert("sub".to_string(), Value::String(args.sub.clone()));
    payload.insert("aud".to_string(), audience_value(&args.aud));
    payload.insert("role".to_string(), Value::String(args.role.clone()));
    payload.insert("exp".to_string(), Value::Number(args.exp.unwrap_or_else(default_exp).into()));

    if let Some(nbf) = args.nbf {
        payload.insert("nbf".to_string(), Value::Number(nbf.into()));
    }
    if let Some(iat) = args.iat {
        payload.insert("iat".to_string(), Value::Number(iat.into()));
    }
    if let Some(jti) = &args.jti {
        payload.insert("jti".to_string(), Value::String(jti.clone()));
    }
    if let Some(claims) = &args.claims {
        merge_claims(&mut payload, claims)?;
    }

    let header_json = serde_json::to_vec(&Value::Object(header))
        .map_err(|err| CliError::InvalidArgument(format!("failed to serialize JWT header: {err}")))?;
    let payload_json = serde_json::to_vec(&Value::Object(payload))
        .map_err(|err| CliError::InvalidArgument(format!("failed to serialize JWT claims: {err}")))?;
    Ok(format!("{}.{}", URL_SAFE_NO_PAD.encode(header_json), URL_SAFE_NO_PAD.encode(payload_json)))
}

/// Serializes JWT audience as a string for one audience or an array for multiple audiences.
fn audience_value(aud: &[String]) -> Value {
    match aud {
        [single] => Value::String(single.clone()),
        _ => Value::Array(aud.iter().cloned().map(Value::String).collect()),
    }
}

/// Merges user-provided custom claims while preventing built-in claim overrides.
fn merge_claims(payload: &mut Map<String, Value>, claims: &str) -> Result<()> {
    let value: Value =
        serde_json::from_str(claims).map_err(|err| CliError::InvalidArgument(format!("invalid claims JSON: {err}")))?;
    let Value::Object(claims) = value else {
        return Err(CliError::InvalidArgument("claims must be a JSON object".to_string()));
    };

    for (key, value) in claims {
        if payload.contains_key(&key) {
            return Err(CliError::InvalidArgument(format!("claim `{key}` conflicts with a built-in JWT claim")));
        }
        payload.insert(key, value);
    }
    Ok(())
}

/// Loads a PEM private key and decrypts it if a passphrase source was requested.
fn load_private_key(args: &GenerateArgs) -> Result<PKey<Private>> {
    let private_key_pem = if let Some(private_key_file) = &args.private_key_file {
        Zeroizing::new(fs::read(private_key_file)?)
    } else if let Ok(private_key) = env::var(PRIVATE_KEY_ENV) {
        Zeroizing::new(private_key.into_bytes())
    } else if let Ok(private_key_file) = env::var(PRIVATE_KEY_FILE_ENV) {
        Zeroizing::new(fs::read(private_key_file)?)
    } else {
        return Err(CliError::Message(format!(
            "missing private key; specify --private-key-file or set {PRIVATE_KEY_ENV}/{PRIVATE_KEY_FILE_ENV}"
        )));
    };

    if let Some(passphrase) = read_private_key_passphrase(args)? {
        return PKey::private_key_from_pem_passphrase(&private_key_pem, passphrase.as_bytes()).map_err(|err| {
            CliError::InvalidArgument(format!(
                "failed to parse encrypted private key PEM: {err}; {SUPPORTED_PRIVATE_KEYS}"
            ))
        });
    }

    PKey::private_key_from_pem(&private_key_pem).map_err(|err| {
        CliError::InvalidArgument(format!(
            "failed to parse private key PEM: {err}; if the private key is encrypted, pass --private-key-passphrase for interactive input or --private-key-passphrase @path to read the passphrase from a file; {SUPPORTED_PRIVATE_KEYS}"
        ))
    })
}

/// Resolves the passphrase source from the optional --private-key-passphrase argument.
fn read_private_key_passphrase(args: &GenerateArgs) -> Result<Option<Zeroizing<String>>> {
    match &args.private_key_passphrase {
        None => Ok(None),
        Some(None) => read_private_key_passphrase_from_input().map(Some),
        Some(Some(value)) => {
            let Some(path) = value.strip_prefix('@') else {
                return Err(CliError::InvalidArgument(
                    "private key passphrase must be provided as --private-key-passphrase @path or entered interactively with --private-key-passphrase".to_string(),
                ));
            };
            let mut passphrase = Zeroizing::new(fs::read_to_string(path)?);
            trim_line_end(&mut passphrase);
            Ok(Some(passphrase))
        },
    }
}

/// Reads a private-key passphrase from the terminal or piped stdin.
fn read_private_key_passphrase_from_input() -> Result<Zeroizing<String>> {
    if io::stdin().is_terminal() {
        rpassword::prompt_password("Private key passphrase: ").map(Zeroizing::new).map_err(CliError::Io)
    } else {
        let mut passphrase = Zeroizing::new(String::new());
        io::stdin().read_to_string(&mut passphrase)?;
        trim_line_end(&mut passphrase);
        Ok(passphrase)
    }
}

/// Removes trailing CR/LF characters from passphrases read from files or stdin.
fn trim_line_end(value: &mut String) {
    while value.ends_with(['\r', '\n']) {
        value.pop();
    }
}

/// Validates that the selected JWT algorithm is compatible with the private key.
fn ensure_key_matches_alg(private_key: &PKey<Private>, alg: &TokenAlg) -> Result<()> {
    let matched = match alg {
        TokenAlg::Rs256 | TokenAlg::Rs384 | TokenAlg::Rs512 | TokenAlg::Ps256 | TokenAlg::Ps384 | TokenAlg::Ps512 => {
            private_key.id() == Id::RSA
        },
        TokenAlg::Es256 => ec_curve_matches(private_key, Nid::X9_62_PRIME256V1)?,
        TokenAlg::Es384 => ec_curve_matches(private_key, Nid::SECP384R1)?,
        TokenAlg::Es512 => ec_curve_matches(private_key, Nid::SECP521R1)?,
        TokenAlg::Sm2 => private_key.id() == Id::SM2 || ec_curve_matches(private_key, Nid::SM2)?,
        TokenAlg::Eddsa => matches!(private_key.id(), Id::ED25519 | Id::ED448),
    };

    if matched {
        Ok(())
    } else {
        Err(CliError::InvalidArgument(format!("private key type does not match alg `{alg}`; {SUPPORTED_PRIVATE_KEYS}")))
    }
}

/// Infers a default JWT algorithm from the private key type and curve.
fn infer_default_alg(private_key: &PKey<Private>) -> Result<TokenAlg> {
    match private_key.id() {
        Id::RSA => Ok(TokenAlg::Ps256),
        Id::SM2 => Ok(TokenAlg::Sm2),
        Id::ED25519 | Id::ED448 => Ok(TokenAlg::Eddsa),
        Id::EC => infer_ec_default_alg(private_key),
        _ => Err(CliError::InvalidArgument(format!(
            "unsupported private key type for JWT signing; {SUPPORTED_PRIVATE_KEYS}"
        ))),
    }
}

/// Infers a default JWT algorithm from the EC curve name.
fn infer_ec_default_alg(private_key: &PKey<Private>) -> Result<TokenAlg> {
    let ec_key = private_key
        .ec_key()
        .map_err(|err| CliError::InvalidArgument(format!("failed to inspect EC private key: {err}")))?;

    match ec_key.group().curve_name() {
        Some(Nid::X9_62_PRIME256V1) => Ok(TokenAlg::Es256),
        Some(Nid::SECP384R1) => Ok(TokenAlg::Es384),
        Some(Nid::SECP521R1) => Ok(TokenAlg::Es512),
        Some(Nid::SM2) => Ok(TokenAlg::Sm2),
        _ => Err(CliError::InvalidArgument(format!(
            "unsupported EC private key curve for JWT signing; {SUPPORTED_PRIVATE_KEYS}"
        ))),
    }
}

/// Checks whether an EC private key uses the expected named curve.
fn ec_curve_matches(private_key: &PKey<Private>, curve: Nid) -> Result<bool> {
    if private_key.id() != Id::EC {
        return Ok(false);
    }

    let ec_key = private_key
        .ec_key()
        .map_err(|err| CliError::InvalidArgument(format!("failed to inspect EC private key: {err}")))?;
    Ok(ec_key.group().curve_name() == Some(curve))
}

/// Signs the JWT signing input with the configured JWA algorithm.
fn sign(signing_input: &[u8], private_key: &PKey<Private>, alg: &TokenAlg) -> Result<Vec<u8>> {
    let mut signer = match alg {
        TokenAlg::Eddsa => Signer::new_without_digest(private_key),
        TokenAlg::Sm2 => Signer::new(MessageDigest::sm3(), private_key),
        _ => Signer::new(message_digest(alg), private_key),
    }
    .map_err(|err| CliError::InvalidArgument(format!("failed to initialize signer: {err}")))?;

    match alg {
        TokenAlg::Rs256 | TokenAlg::Rs384 | TokenAlg::Rs512 => {
            // RS* JWT algorithms use RSASSA-PKCS1-v1_5 with the matching SHA digest.
            signer
                .set_rsa_padding(Padding::PKCS1)
                .map_err(|err| CliError::InvalidArgument(format!("failed to configure RSA padding: {err}")))?;
        },
        TokenAlg::Ps256 | TokenAlg::Ps384 | TokenAlg::Ps512 => {
            // PS* JWT algorithms require RSA-PSS with MGF1 using the same digest and salt length equal to digest length.
            signer
                .set_rsa_padding(Padding::PKCS1_PSS)
                .map_err(|err| CliError::InvalidArgument(format!("failed to configure RSA-PSS padding: {err}")))?;
            signer
                .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
                .map_err(|err| CliError::InvalidArgument(format!("failed to configure RSA-PSS salt length: {err}")))?;
            signer
                .set_rsa_mgf1_md(message_digest(alg))
                .map_err(|err| CliError::InvalidArgument(format!("failed to configure RSA-PSS MGF1 digest: {err}")))?;
        },
        _ => {},
    }

    let signature = if matches!(alg, TokenAlg::Eddsa) {
        signer
            .sign_oneshot_to_vec(signing_input)
            .map_err(|err| CliError::InvalidArgument(format!("failed to sign JWT: {err}")))
    } else {
        signer.update(signing_input).map_err(|err| CliError::InvalidArgument(format!("failed to sign JWT: {err}")))?;
        signer.sign_to_vec().map_err(|err| CliError::InvalidArgument(format!("failed to sign JWT: {err}")))
    }?;

    if matches!(alg, TokenAlg::Es256 | TokenAlg::Es384 | TokenAlg::Es512) {
        // OpenSSL returns DER-encoded ECDSA signatures; JWA requires fixed-width raw R || S bytes.
        ecdsa_der_to_jwa(&signature, ecdsa_component_len(alg))
    } else {
        Ok(signature)
    }
}

/// Converts a DER-encoded OpenSSL ECDSA signature to JWA raw R || S format.
fn ecdsa_der_to_jwa(signature: &[u8], component_len: usize) -> Result<Vec<u8>> {
    let signature = EcdsaSig::from_der(signature)
        .map_err(|err| CliError::InvalidArgument(format!("failed to parse ECDSA signature DER: {err}")))?;
    let mut raw = signature
        .r()
        .to_vec_padded(component_len as i32)
        .map_err(|err| CliError::InvalidArgument(format!("failed to encode ECDSA signature r: {err}")))?;
    raw.extend(
        signature
            .s()
            .to_vec_padded(component_len as i32)
            .map_err(|err| CliError::InvalidArgument(format!("failed to encode ECDSA signature s: {err}")))?,
    );
    Ok(raw)
}

/// Returns the fixed byte width for each ECDSA signature component.
fn ecdsa_component_len(alg: &TokenAlg) -> usize {
    match alg {
        TokenAlg::Es256 => 32,
        TokenAlg::Es384 => 48,
        TokenAlg::Es512 => 66,
        _ => 0,
    }
}

/// Returns the OpenSSL digest associated with each hash-based JWT algorithm.
fn message_digest(alg: &TokenAlg) -> MessageDigest {
    match alg {
        TokenAlg::Rs256 | TokenAlg::Ps256 | TokenAlg::Es256 => MessageDigest::sha256(),
        TokenAlg::Rs384 | TokenAlg::Ps384 | TokenAlg::Es384 => MessageDigest::sha384(),
        TokenAlg::Rs512 | TokenAlg::Ps512 | TokenAlg::Es512 => MessageDigest::sha512(),
        TokenAlg::Sm2 => MessageDigest::sm3(),
        TokenAlg::Eddsa => MessageDigest::sha512(),
    }
}

/// Calculates the default expiration time as now plus one hour.
fn default_exp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() + DEFAULT_EXP_AFTER_SECONDS)
        .unwrap_or(DEFAULT_EXP_AFTER_SECONDS)
}

#[cfg(test)]
mod tests {
    use super::{generate_token, GenerateArgs, TokenAlg, TokenCli, TokenCommand};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use clap::Parser;
    use openssl::ec::{EcGroup, EcKey};
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use serde_json::Value;
    use std::fs;

    /// Verifies parsing of all supported token generate arguments.
    #[test]
    fn parses_token_generate_command() {
        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            command: Command,
        }

        #[derive(clap::Subcommand)]
        enum Command {
            Token(TokenCli),
        }

        let wrapper = Wrapper::parse_from([
            "rbs-cli",
            "token",
            "generate",
            "--private-key-file",
            "key.pem",
            "--private-key-passphrase",
            "@passphrase.txt",
            "--iss",
            "test-iss",
            "--sub",
            "test-sub",
            "--aud",
            "test-aud",
            "--aud",
            "test-aud-2",
            "--role",
            "test-role",
            "--exp",
            "1893456000",
            "--nbf",
            "1893452400",
            "--iat",
            "1893452400",
            "--jti",
            "test-jti",
            "--alg",
            "RS256",
            "--kid",
            "test-kid",
            "--claims",
            "{\"role\":\"admin\",\"env\":\"test\"}",
        ]);

        match wrapper.command {
            Command::Token(TokenCli {
                command:
                    TokenCommand::Generate(GenerateArgs {
                        private_key_file,
                        private_key_passphrase,
                        iss,
                        sub,
                        aud,
                        role,
                        exp,
                        nbf,
                        iat,
                        jti,
                        alg,
                        kid,
                        claims,
                    }),
            }) => {
                assert_eq!(private_key_file.as_deref(), Some("key.pem"));
                assert_eq!(private_key_passphrase.as_ref().and_then(Option::as_deref), Some("@passphrase.txt"));
                assert_eq!(iss, "test-iss");
                assert_eq!(sub, "test-sub");
                assert_eq!(aud, vec!["test-aud".to_string(), "test-aud-2".to_string()]);
                assert_eq!(role, "test-role");
                assert_eq!(exp, Some(1893456000));
                assert_eq!(nbf, Some(1893452400));
                assert_eq!(iat, Some(1893452400));
                assert_eq!(jti.as_deref(), Some("test-jti"));
                assert_eq!(alg, Some(TokenAlg::Rs256));
                assert_eq!(kid.as_deref(), Some("test-kid"));
                assert_eq!(claims.as_deref(), Some("{\"role\":\"admin\",\"env\":\"test\"}"));
            },
        }
    }

    /// Verifies that --private-key-passphrase without a value requests interactive input.
    #[test]
    fn parses_private_key_passphrase_prompt_request() {
        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            command: Command,
        }

        #[derive(clap::Subcommand)]
        enum Command {
            Token(TokenCli),
        }

        let wrapper = Wrapper::parse_from(["rbs-cli", "token", "generate", "--private-key-passphrase"]);

        match wrapper.command {
            Command::Token(TokenCli {
                command: TokenCommand::Generate(GenerateArgs { private_key_passphrase, .. }),
            }) => {
                assert_eq!(private_key_passphrase, Some(None));
            },
        }
    }

    /// Verifies clap/default construction for optional token generate arguments.
    #[test]
    fn parses_token_generate_defaults() {
        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            command: Command,
        }

        #[derive(clap::Subcommand)]
        enum Command {
            Token(TokenCli),
        }

        let wrapper = Wrapper::parse_from(["rbs-cli", "token", "generate"]);

        match wrapper.command {
            Command::Token(TokenCli {
                command: TokenCommand::Generate(GenerateArgs { iss, sub, aud, role, exp, alg, .. }),
            }) => {
                assert_eq!(iss, "rbs-cli");
                assert_eq!(sub, "administrator");
                assert_eq!(aud, vec!["globaltrustauthority-rbs".to_string()]);
                assert_eq!(role, "admin");
                assert_eq!(exp, None);
                assert_eq!(alg, None);
            },
        }
    }

    /// Verifies Ed25519 keys produce a three-part EdDSA JWT.
    #[test]
    fn generate_token_signs_with_ed25519_key_file() {
        let key = PKey::generate_ed25519().unwrap();
        let key_pem = key.private_key_to_pem_pkcs8().unwrap();
        let key_path = std::env::temp_dir().join(format!("rbs-cli-test-{}-ed25519.pem", std::process::id()));
        fs::write(&key_path, key_pem).unwrap();

        let args = GenerateArgs {
            private_key_file: Some(key_path.to_string_lossy().to_string()),
            exp: Some(1893456000),
            ..Default::default()
        };

        let token = generate_token(&args).unwrap();
        fs::remove_file(&key_path).unwrap();
        let header = decode_token_header(&token.token);

        assert_eq!(token.token.split('.').count(), 3);
        assert_eq!(header.get("alg").and_then(Value::as_str), Some("EdDSA"));
    }

    /// Verifies ES256 signatures are emitted in JWA raw R || S format.
    #[test]
    fn generate_token_signs_es256_as_jwa_raw_signature() {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let key = PKey::from_ec_key(EcKey::generate(&group).unwrap()).unwrap();
        let key_pem = key.private_key_to_pem_pkcs8().unwrap();
        let key_path = std::env::temp_dir().join(format!("rbs-cli-test-{}-es256.pem", std::process::id()));
        fs::write(&key_path, key_pem).unwrap();

        let args = GenerateArgs {
            private_key_file: Some(key_path.to_string_lossy().to_string()),
            exp: Some(1893456000),
            alg: Some(TokenAlg::Es256),
            ..Default::default()
        };

        let token = generate_token(&args).unwrap();
        fs::remove_file(&key_path).unwrap();
        let signature = token.token.split('.').nth(2).unwrap();
        let signature = URL_SAFE_NO_PAD.decode(signature).unwrap();

        assert_eq!(signature.len(), 64);
    }

    /// Verifies P-256 EC keys default to ES256 when --alg is omitted.
    #[test]
    fn generate_token_infers_es256_from_p256_key() {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let key = PKey::from_ec_key(EcKey::generate(&group).unwrap()).unwrap();
        let key_pem = key.private_key_to_pem_pkcs8().unwrap();
        let key_path = std::env::temp_dir().join(format!("rbs-cli-test-{}-infer-es256.pem", std::process::id()));
        fs::write(&key_path, key_pem).unwrap();

        let args = GenerateArgs {
            private_key_file: Some(key_path.to_string_lossy().to_string()),
            exp: Some(1893456000),
            ..Default::default()
        };

        let token = generate_token(&args).unwrap();
        fs::remove_file(&key_path).unwrap();
        let header = decode_token_header(&token.token);

        assert_eq!(header.get("alg").and_then(Value::as_str), Some("ES256"));
    }

    /// Decodes the JWT header from a generated token for assertions.
    fn decode_token_header(token: &str) -> Value {
        let header = token.split('.').next().unwrap();
        let header = URL_SAFE_NO_PAD.decode(header).unwrap();
        serde_json::from_slice(&header).unwrap()
    }
}
