use ldap3::result::Result;
use ldap3::{LdapConn, Scope, SearchEntry};

use crate::Host;

pub fn ldap_checker(h: Host) -> Result<()> {
    let mut ldap = LdapConn::new("ldap://localhost:2389")?;
    ldap.simple_bind(&h.bind_dn, &h.bind_pw)?;

    let (rs, _res) = ldap
        .search(&h.base, Scope::Subtree, &h.filter, vec!["*"])?
        .success()?;
    for entry in rs {
        println!("{:?}", SearchEntry::construct(entry));
    }
    Ok(ldap.unbind()?)
}
