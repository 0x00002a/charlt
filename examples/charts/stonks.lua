local STONKS_VALUES = {}
for n = 0, 500 do
    table.insert(STONKS_VALUES, { x = n, y = n ^ 2 })
end

return {
    type = "xy-scatter",
    datasets = {
        {
            name = "stonks",
            values = STONKS_VALUES,
        }
    },
    axis = { y = 'money (£)', x = 'time (h)' },
}
