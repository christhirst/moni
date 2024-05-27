use ldap3::result::Result;
use ldap3::{LdapConn, Scope, SearchEntry};

use crate::Host;

fn www(h: Host) -> Result<()> {
    let mut ldap = LdapConn::new("ldap://localhost:2389")?;
    ldap.simple_bind(&h.bind_dn, &h.bind_pw)?;

    let (rs, _res) = ldap
        .search(
            "ou=Places,dc=example,dc=org",
            Scope::Subtree,
            "(&(objectClass=locality)(l=ma*))",
            vec!["l"],
        )?
        .success()?;
    for entry in rs {
        println!("{:?}", SearchEntry::construct(entry));
    }
    Ok(ldap.unbind()?)
}
