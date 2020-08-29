# BetterSign

## Todo

- [x] Initialize the repo and set up initial structure.
- [x] Add the LICENSE file.
- [x] Rough draft of the README.md.
- [x] Create the initial BetterSign lib and executable driver.
- [ ] Add error_chain crate to simplify the custom error types.
- [ ] Switch to using background-jobs crate to handle the job queueing.
- [ ] Add support for r/w of Linked Data Signature format signatures.
- [ ] Add support for r/w of JWT/JWS format signatures.
- [ ] Define the status keywords and parameters for Git interface.

## Introduction

BetterSign (`bs`) is a new signing tool designed to streamline the generation
and verification of signed manifests for files and data. The goal is to create
a better code and release signing tool that integrates seamlessly with Git and
provides real value to digitally signing commits through a new strategy for key
management.

There are many problems with Git's reliance on GPG for its sole signing tool.
The problem is that most people who clone a repo do not have all of the public
keys of the commit signers nor do they want to spend the time it takes to
manually download the public keys from key servers. Even if find a way around
the difficult task of generating a list of key IDs from the repo and they
download the public keys, they can't necessarily trust that the keys are the
real keys used by the commit signers.

The solution to this problem is to store the "keyring" of public keys in the
Git repo itself and to track the keys as you would any source code file in the
repo. This gives a repo the ability to self-validate the signatures in the repo
log. It also allows for the revocation of keys through simply deleting the key
from the repo. With simple Git hooks, rules such as "all commits must be signed
by a key in the repo" are trivial to implement and help enforce the provenance
regime.

So why a new signing tool? Why not just use GPG? We could use GPG keyrings
stored in the repo, however GPG keyrings are binary blobs and key management
with a tracked repo (e.g. adding a new key) would always result in a new copy
of the keyring and it would be difficult to inspect. The BetterSign tool is
design to understand decentralized identifier documents (DID docs) that contain
the key material and any other information associated with a contributor
identity. Typically, DID docs are formatted in JSON and are therefore both
human and machine readable and easy to manage as tracked files in a repo.

BetterSign uses a simple organization of DID documents in a Git repo that is
inspired by the Maildir standard. The unimaginative name is the DIDdir format
and it is specified in the [W3C DID Git Method
specification](https://github.com/dhuseby/did-git-spec/blob/master/did-git-spec.md).
The DID Git method specification also defines the standard way to reference a
given Git repo and a specific identity stored within it called a decentralized
identifier (DID). In the case of identities stored in a Git repo, the DID
begins with "did:git". You will pass DID's to BetterSign as the arguments
specifying the identities for the different operations (e.g. "sign", "verify").

BetterSign is designed to be used for managing identities and key material for
contributors to open source projects but it is also useful as a generic signing
tool as well. By default BetterSign creates a DIDdir Git repo in your home
folder similar to the way GPG does. That is your personal keyring and the
key material in there may contain secrets like your private keys just like a
GPG keyring does.

## User Interface

BetterSign is implemented as a command line tool called `bs`. The interface is
rather simple and follows the pattern of: `bs <subcommand> [options]`. In all
cases, BetterSign uses the DIDdir in the user's home folder unless the
`--keyring` command line option is used. It also uses the identity in the DID
document with the "default" alias unless the `--did` option is used with a
valid DID. See the DID Git Method specification linked to above for details on
identity aliases.

BetterSign also supports outputting machine parseable status on a given file
descriptor specified by the `--status-fd` command line option. The format of
the output is similar to what GPG outputs. Each line begins with "[BS:] "
followed by a status keyword (e.g. DID_CONSIDERED, etc) followed by the
parameters for the keywork, if any.

## Sign

The `sign` subcommand generates a detached digital signature over the given
file(s) or data piped over stdin. To generate a signature over one or more
files, execute `bs` like so:

```
$ bs sign [options] [<file> ...]
```

To generate a signature for data piped to BetterSign over stdin, use `-`
instead of the file name(s). In addition to the `--keyring` and `--did`
options, the `sign` subcommand also supports a `--format` subcommand for
specifying the format of the resulting signature. The supported values are
`lds` for the [Linked Data Signature
(LDS)](https://w3c-dvcg.github.io/ld-signatures/) format and `jwt` for the
[JSON Web Token (JWT)](https://tools.ietf.org/html/rfc7519) format.

BetterSign uses the EdDSA algorithm when signing data. It is the combination
of the SHA-512 digest algorithm and the Edwards curve encryption algorithm.
When the output is in LDS format, the signature type value is
"Ed25519Signature2018". When the output is in JWT format, BetterSign uses the
*non-standard* "alg" param value of "ED512" meaning EdDSA. The current standard
set of "alg" values is defined in the [JSON Web Algorithms RFC 7518
§3.1](https://tools.ietf.org/html/rfc7518#section-3.1) and does not contain any
Ed25519 based signature schemes so BetterSign is doing...uh...better.

When used with Git to sign commits, Git will pipe the data to be signed over
stdin to BetterSign. The resulting signature, by default, is in LDS format.
When signing files and the output format is LDS format BetterSign outputs a a
linked data signature file with a non-standard "files" attribute inside of the
proof and part of the data that is signed. The "files" attribute is an array of
JSON objects with a single key consisting of the file name and the value being
the digest value of the file like so:

```
{
  "@context": "https://w3id.org/identity/v1",
  "proof": {
    "type": "Ed25519Signature2018",
    "creator": "did:git:...",
    "created": "1970-01-01T00:00:00Z",
    "nonce": "...",
    "proofValue": "...",
    "files": [
      { "foo.txt": "..." },
      { "bar.txt", "..." },
      { "baz.txt", "..." }
    ]
  }
}
```

This serves as a manifest file for the authentication of the files included in
the signature.

## Verify

The `verify` subcommand takes a signature file in either LDS or JWT format and
attempts to verify the data signed. If the signature file does not contain a
"files" attribute in the "proof" it is assumed that the data that was signed
will be piped over stdin.

If the signature file does contain a "files" attribute in the "proof" then
the listed files will be found and run through the digest algorithm and checked
to see if they have been modified or not before checking that the signature is
valid.

An important detail to point out is that the identities in the DIDdir will
change over time and the historical context is necessary to be able to find the
correct DID document and key material needed to verify the signature. For that
BetterSign relies on the position in the repo history to derive the correct
state of the DIDdir before resolving the "creator" DID into the correct DID
document and extracting the key material for the the signature verification.
BetterSign does not need to worry about this detail as it is handle by the
DIDdir library, but it something to be aware of to get your mental model
correct.

## Notes on Git

The current Git commit signing system is hard coded to use GPG/GPGSM and
supports both PGP identities with GPG and x.509 identities with GPGSM. Git
relies entirely on external tools to sign and verify data. Git only handles
storing/extracting the signatures in/from the commit meta data and passing it
to GPG.

BetterSign has an interface that is somewhat similar to GPG to make the
modifications to Git simpler. BetterSign supports specifying a <key-id>

### Signature Creation

The important part here is the way Git interfaces with external tools. When it
shells out to GPG to create a signature it pipes the content to be signed to
GPG over stdin and reads back the signature over stdout and the machine
parseable status over stderr (e.g. fd = 2).

#### Commits

0. If the sign commit flag is passed to Git (e.g. `git commit -S`) the
   do_sign_commit function (commit.c around line 937) which drives the whole
   process.
1. If the key-id is not passed to the command line (e.g. `git commit
   --gpg-sign=<key-id>`) then do_sign_commit calls the get_signing_key function
   (commit.c around line 951).
2. The get_signing_key function will return the signing key set up when the
   git_gpg_config function (gpg-interface.c) is called from the porcelain setup
   or it will return the committer's name and email. The git_gpg_config
   function parses the .gitconfig for the gpg.* config keys and if the
   user.signing key setting exists, the key id will be stored and later
   returned from get_signing_key.
3. The do_sign_commit now calls sign_buffer (commit.c around line 952) to sign
   the commit information (e.g. hash of parent commits, author, committer,
   encoding, extra header information, then a newline followed by the commit
   message).
4. The sign_buffer function initializes the child_process struct with the
   correct executable name and parameters to run the external tool to sign the
   data and to get a machine parseable status back.
5. When running GPG, Git uses teh following command line to sign a commit:
   `gpg --status-fd=2 -bsau <key-id>`. The `-b` option tells GPG to create a
   "detached" signature that only contains the signature data and not the data
   that was signed. The `-s` option tells GPG to do the sign operation. The
   `-a` option tells GPG to "armor" the output as ASCII instead of binary. The
   `-u` option tells GPG that the <key-id> follows the `-u` option specifying
   which key to use to sign the data.
6. The sign_buffer function then executes the child process gathering the
   resulting signature and status information. It does a string match against
   the status data looking for "\n[GNUPG:] SIG_CREATED " to determine if the
   signing operation succeeded.
7. If the signing was successful, the do_sign_commit function then inserts the
   signature as a "gpgsig" header in the commit data.

#### Tags

0. Signing tags is much more simple. If the sign flag `-s` is passed to the
   `git tag` command, after the tag is built, it is passed to the sign_buffer
   function. Currently the `git tag` command does not support specifying the
   <key-id> on the command line so step 1 in the Commits section above is not
   done. Only the git_gpg_config function is called to load and store the
   correct signing key id before calling the sign_buffer function.
1. The steps 3-6 in the above Commits section are the same.
2. After the sign_buffer returns, if it was successful, the signature data is
   simply appended to the end of the tag data.

### Signature Verification

The important part here is the way Git interfaces with external tools. When it
shells out to GPG, it first writes the signature to a temporary file and then
pipes the signed content to GPG over stdin and reads back the machine parseable
status over stdout. Below is a detailed explaination of the execution flow for
verifying signed commits and tags.

#### Commits

0. Commit verification starts in commit.c in the check_commit_signature
   function.
1. It first parses the commit buffer looking for the "gpgsig" start sigil and
   then extracts the signature data up to the next empty line in the commit
   buffer (commit.c parse_signed_commit).
2. It then calls check_signature passing the commit buffer and the signature
   buffer (commit.c around line 1099). This is the entry point into the GPG
   infrastructure code in gpg-interface.c.
3. The check_signature function initialized the signature check structure
   members and then calls verify_signed_buffer (gpg-interface.c around line
   196).
4. The verify_signed_buffer first creates a temporary file (mkstemp) and saves
   the signature to the file.
5. The verify_signed_buffer then calls get_format_by_sig which does string
   matching against the signature buffer to determine if the signature is a GPG
   signature ("-----BEGIN PGP SIGNATURE-----") or a GPGSM signature
   ("-----BEGIN PGP MESSAGE-----").
6. The verify_signed_buffer initializes a child_process struct with the correct
   executable name and parameters to run the external tool to verify the
   signature and get the machine parseable response.
7. When running GPG, Git uses the following command line:
   `gpg --keyid-format=long --status-fd=1 --verify <tempfile with signature> -`
   The code pipe-forks using Git's child_process infrastructure and pipes the
   commit buffer to the child process' stdin and reads the machine parseable
   status outputs from the child process' stdout (--status-fd=1).
8. The verify_signed_buffer then checks for the GPG string "\n[GNUPG:] GOODSIG "
   to set the value of the return status to 1/true if the string exists.
9. The check_signature finishes by calling parse_gpg_output to parse the status
   output into a signature status result character (e.g. 'U', 'E', 'N') that
   mean different things. The meaning is listed in gpg-interface.c around line
   97. It also parses out the signature verification lines that show the
   creator and the date and the status so that they can be output to the log if
   needed.
A. The stack unwinds back to commit.c

#### Tags

0. Tag verification starts in tag.c in the gpg_verify_tag function.
1. It first reads the oid_object_info to make sure the object is a tag before
   reading the object file into memory (tag.c around line 47).
2. It then runs run_gpg_verify which first calls parse_signature which searches
   for the GPG/GPGSM signature begin strings (e.g. "-----BEGIN PGP SIGNATURE-----")
   and extracts the signature up to the first empty line after the signature
   begin string.
3. Then the run_gpg_verify calls the check_signature function passing the
   tag contents buffer and the signature buffer (tag.c line 29). This is the
   main entry point into the GPG infrastructure code in gpg-interface.c.
4. The execution then follows steps 3-9 in the Commit section above.
5. The stack unwinds back to tag.c where the last thing that happens is
   printing the signature check status if that was asked for.

