local csv = {}

function csv.parse_file(f, opts)
    local content = f:read("*a")
    return __rs.csv.parse_string(content, opts)
end

return {
    namespace = "csv",
    module = csv
}
