descr = "ldap w/ search"

function verify(user, password)
    return ldap_search_bind("ldaps://ldap.example.com/",
        -- the user we use to find the correct DN
        "cn=search_user,ou=users,dc=example,dc=com", "searchpw",
        -- base DN we search in
        "dc=example,dc=com",
        -- the user we test
        user, password)
end
