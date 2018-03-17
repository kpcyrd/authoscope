descr = "ldap"

function verify(user, password)
    return ldap_bind("ldaps://ldap.example.com/",
        "cn=\"" .. ldap_escape(user) .. "\",ou=users,dc=example,dc=com", password)
end
