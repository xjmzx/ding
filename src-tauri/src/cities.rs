//! Bundled cities: name -> pinned coordinates + a label sized for the
//! weather tile. Copied from pong's _CITY_PRESETS. Pinned rather than
//! geocoded: a preset needs no network, its label is known to fit, and an
//! ambiguous name cannot drift (the geocoder resolves "La Paz" to Mexico,
//! not Bolivia). Anything not listed here is geocoded online instead.

pub struct City {
    pub name: &'static str,
    pub label: &'static str,
    pub lat: f64,
    pub lon: f64,
}

pub const REGIONS: &[(&str, &[City])] = &[
    (
        "Africa",
        &[
            City { name: "Accra", label: "ACCRA", lat: 5.6037, lon: -0.187 },
            City { name: "Addis Ababa", label: "ADDIS", lat: 9.025, lon: 38.7469 },
            City { name: "Algiers", label: "ALGIERS", lat: 36.7538, lon: 3.0588 },
            City { name: "Cairo", label: "CAIRO", lat: 30.0444, lon: 31.2357 },
            City { name: "Cape Town", label: "CAPETWN", lat: -33.9249, lon: 18.4241 },
            City { name: "Casablanca", label: "CASA", lat: 33.5731, lon: -7.5898 },
            City { name: "Dakar", label: "DAKAR", lat: 14.7167, lon: -17.4677 },
            City { name: "Dar es Salaam", label: "DAR", lat: -6.7924, lon: 39.2083 },
            City { name: "Johannesburg", label: "JOBURG", lat: -26.2041, lon: 28.0473 },
            City { name: "Kampala", label: "KAMPALA", lat: 0.3476, lon: 32.5825 },
            City { name: "Kinshasa", label: "KINSHASA", lat: -4.4419, lon: 15.2663 },
            City { name: "Lagos", label: "LAGOS", lat: 6.5244, lon: 3.3792 },
            City { name: "Luanda", label: "LUANDA", lat: -8.839, lon: 13.2894 },
            City { name: "Marrakesh", label: "MARRKSH", lat: 31.6295, lon: -7.9811 },
            City { name: "Nairobi", label: "NAIROBI", lat: -1.2864, lon: 36.8172 },
            City { name: "Tunis", label: "TUNIS", lat: 36.8065, lon: 10.1815 },
        ],
    ),
    (
        "Asia",
        &[
            City { name: "Almaty", label: "ALMATY", lat: 43.222, lon: 76.8512 },
            City { name: "Bangkok", label: "BANGKOK", lat: 13.7563, lon: 100.5018 },
            City { name: "Beijing", label: "BEIJING", lat: 39.9042, lon: 116.4074 },
            City { name: "Bengaluru", label: "BENGLRU", lat: 12.9716, lon: 77.5946 },
            City { name: "Chennai", label: "CHENNAI", lat: 13.0827, lon: 80.2707 },
            City { name: "Colombo", label: "COLOMBO", lat: 6.9271, lon: 79.8612 },
            City { name: "Delhi", label: "DELHI", lat: 28.7041, lon: 77.1025 },
            City { name: "Dhaka", label: "DHAKA", lat: 23.8103, lon: 90.4125 },
            City { name: "Hanoi", label: "HANOI", lat: 21.0278, lon: 105.8342 },
            City { name: "Ho Chi Minh City", label: "SAIGON", lat: 10.8231, lon: 106.6297 },
            City { name: "Hong Kong", label: "HONGKNG", lat: 22.3193, lon: 114.1694 },
            City { name: "Jakarta", label: "JAKARTA", lat: -6.2088, lon: 106.8456 },
            City { name: "Karachi", label: "KARACHI", lat: 24.8607, lon: 67.0011 },
            City { name: "Kathmandu", label: "KTMANDU", lat: 27.7172, lon: 85.324 },
            City { name: "Kolkata", label: "KOLKATA", lat: 22.5726, lon: 88.3639 },
            City { name: "Kuala Lumpur", label: "KL", lat: 3.139, lon: 101.6869 },
            City { name: "Lahore", label: "LAHORE", lat: 31.5204, lon: 74.3587 },
            City { name: "Manila", label: "MANILA", lat: 14.5995, lon: 120.9842 },
            City { name: "Mumbai", label: "MUMBAI", lat: 19.076, lon: 72.8777 },
            City { name: "Osaka", label: "OSAKA", lat: 34.6937, lon: 135.5023 },
            City { name: "Phnom Penh", label: "PPENH", lat: 11.5564, lon: 104.9282 },
            City { name: "Seoul", label: "SEOUL", lat: 37.5665, lon: 126.978 },
            City { name: "Shanghai", label: "SHANGHAI", lat: 31.2304, lon: 121.4737 },
            City { name: "Singapore", label: "SNGPORE", lat: 1.3521, lon: 103.8198 },
            City { name: "Taipei", label: "TAIPEI", lat: 25.033, lon: 121.5654 },
            City { name: "Tashkent", label: "TASHKENT", lat: 41.2995, lon: 69.2401 },
            City { name: "Tokyo", label: "TOKYO", lat: 35.6762, lon: 139.6503 },
            City { name: "Ulaanbaatar", label: "ULNBTR", lat: 47.8864, lon: 106.9057 },
            City { name: "Vientiane", label: "VNTIANE", lat: 17.9757, lon: 102.6331 },
            City { name: "Yangon", label: "YANGON", lat: 16.8409, lon: 96.1735 },
        ],
    ),
    (
        "Europe",
        &[
            City { name: "Amsterdam", label: "AMSTRDM", lat: 52.3676, lon: 4.9041 },
            City { name: "Athens", label: "ATHENS", lat: 37.9838, lon: 23.7275 },
            City { name: "Barcelona", label: "BARCLNA", lat: 41.3851, lon: 2.1734 },
            City { name: "Belgrade", label: "BELGRAD", lat: 44.7866, lon: 20.4489 },
            City { name: "Berlin", label: "BERLIN", lat: 52.52, lon: 13.405 },
            City { name: "Brussels", label: "BRUSSELS", lat: 50.8503, lon: 4.3517 },
            City { name: "Bucharest", label: "BUCHRST", lat: 44.4268, lon: 26.1025 },
            City { name: "Budapest", label: "BUDPEST", lat: 47.4979, lon: 19.0402 },
            City { name: "Copenhagen", label: "CPNHGN", lat: 55.6761, lon: 12.5683 },
            City { name: "Dublin", label: "DUBLIN", lat: 53.3498, lon: -6.2603 },
            City { name: "Edinburgh", label: "EDINB", lat: 55.9533, lon: -3.1883 },
            City { name: "Geneva", label: "GENEVA", lat: 46.2044, lon: 6.1432 },
            City { name: "Glasgow", label: "GLASGOW", lat: 55.8642, lon: -4.2518 },
            City { name: "Hamburg", label: "HAMBURG", lat: 53.5511, lon: 9.9937 },
            City { name: "Helsinki", label: "HELSNKI", lat: 60.1699, lon: 24.9384 },
            City { name: "Istanbul", label: "ISTNBUL", lat: 41.0082, lon: 28.9784 },
            City { name: "Kyiv", label: "KYIV", lat: 50.4501, lon: 30.5234 },
            City { name: "Lisbon", label: "LISBON", lat: 38.7223, lon: -9.1393 },
            City { name: "London", label: "LONDON", lat: 51.5074, lon: -0.1278 },
            City { name: "Madrid", label: "MADRID", lat: 40.4168, lon: -3.7038 },
            City { name: "Manchester", label: "MANCHR", lat: 53.4808, lon: -2.2426 },
            City { name: "Milan", label: "MILAN", lat: 45.4642, lon: 9.19 },
            City { name: "Munich", label: "MUNICH", lat: 48.1351, lon: 11.582 },
            City { name: "Oslo", label: "OSLO", lat: 59.9139, lon: 10.7522 },
            City { name: "Paris", label: "PARIS", lat: 48.8566, lon: 2.3522 },
            City { name: "Porto", label: "PORTO", lat: 41.1579, lon: -8.6291 },
            City { name: "Prague", label: "PRAGUE", lat: 50.0755, lon: 14.4378 },
            City { name: "Reykjavik", label: "RKJAVIK", lat: 64.1466, lon: -21.9426 },
            City { name: "Riga", label: "RIGA", lat: 56.9496, lon: 24.1052 },
            City { name: "Rome", label: "ROME", lat: 41.9028, lon: 12.4964 },
            City { name: "Sofia", label: "SOFIA", lat: 42.6977, lon: 23.3219 },
            City { name: "Stockholm", label: "STCKHLM", lat: 59.3293, lon: 18.0686 },
            City { name: "Tallinn", label: "TALLINN", lat: 59.437, lon: 24.7536 },
            City { name: "Vienna", label: "VIENNA", lat: 48.2082, lon: 16.3738 },
            City { name: "Vilnius", label: "VILNIUS", lat: 54.6872, lon: 25.2797 },
            City { name: "Warsaw", label: "WARSAW", lat: 52.2297, lon: 21.0122 },
            City { name: "Zagreb", label: "ZAGREB", lat: 45.815, lon: 15.9819 },
            City { name: "Zurich", label: "ZURICH", lat: 47.3769, lon: 8.5417 },
        ],
    ),
    (
        "Middle East",
        &[
            City { name: "Abu Dhabi", label: "ABUDHABI", lat: 24.4539, lon: 54.3773 },
            City { name: "Amman", label: "AMMAN", lat: 31.9454, lon: 35.9284 },
            City { name: "Baghdad", label: "BAGHDAD", lat: 33.3152, lon: 44.3661 },
            City { name: "Beirut", label: "BEIRUT", lat: 33.8938, lon: 35.5018 },
            City { name: "Doha", label: "DOHA", lat: 25.2854, lon: 51.531 },
            City { name: "Dubai", label: "DUBAI", lat: 25.2048, lon: 55.2708 },
            City { name: "Jeddah", label: "JEDDAH", lat: 21.4858, lon: 39.1925 },
            City { name: "Jerusalem", label: "JRSLEM", lat: 31.7683, lon: 35.2137 },
            City { name: "Kuwait City", label: "KUWAIT", lat: 29.3759, lon: 47.9774 },
            City { name: "Manama", label: "MANAMA", lat: 26.2285, lon: 50.586 },
            City { name: "Muscat", label: "MUSCAT", lat: 23.588, lon: 58.3829 },
            City { name: "Riyadh", label: "RIYADH", lat: 24.7136, lon: 46.6753 },
            City { name: "Tehran", label: "TEHRAN", lat: 35.6892, lon: 51.389 },
            City { name: "Tel Aviv", label: "TELAVIV", lat: 32.0853, lon: 34.7818 },
        ],
    ),
    (
        "North America",
        &[
            City { name: "Anchorage", label: "ANCHRGE", lat: 61.2181, lon: -149.9003 },
            City { name: "Atlanta", label: "ATLANTA", lat: 33.749, lon: -84.388 },
            City { name: "Austin", label: "AUSTIN", lat: 30.2672, lon: -97.7431 },
            City { name: "Boston", label: "BOSTON", lat: 42.3601, lon: -71.0589 },
            City { name: "Calgary", label: "CALGARY", lat: 51.0447, lon: -114.0719 },
            City { name: "Chicago", label: "CHICAGO", lat: 41.8781, lon: -87.6298 },
            City { name: "Dallas", label: "DALLAS", lat: 32.7767, lon: -96.797 },
            City { name: "Denver", label: "DENVER", lat: 39.7392, lon: -104.9903 },
            City { name: "Detroit", label: "DETROIT", lat: 42.3314, lon: -83.0458 },
            City { name: "Guadalajara", label: "GUADLJR", lat: 20.6597, lon: -103.3496 },
            City { name: "Guatemala City", label: "GUATMLA", lat: 14.6349, lon: -90.5069 },
            City { name: "Havana", label: "HAVANA", lat: 23.1136, lon: -82.3666 },
            City { name: "Honolulu", label: "HNLULU", lat: 21.3069, lon: -157.8583 },
            City { name: "Houston", label: "HOUSTON", lat: 29.7604, lon: -95.3698 },
            City { name: "Kingston", label: "KINGSTON", lat: 17.9714, lon: -76.7928 },
            City { name: "Las Vegas", label: "VEGAS", lat: 36.1699, lon: -115.1398 },
            City { name: "Los Angeles", label: "LA", lat: 34.0522, lon: -118.2437 },
            City { name: "Mexico City", label: "MEXICO", lat: 19.4326, lon: -99.1332 },
            City { name: "Miami", label: "MIAMI", lat: 25.7617, lon: -80.1918 },
            City { name: "Minneapolis", label: "MNPOLIS", lat: 44.9778, lon: -93.265 },
            City { name: "Monterrey", label: "MNTRREY", lat: 25.6866, lon: -100.3161 },
            City { name: "Montreal", label: "MONTRL", lat: 45.5017, lon: -73.5673 },
            City { name: "New York", label: "NEWYORK", lat: 40.7128, lon: -74.006 },
            City { name: "Ottawa", label: "OTTAWA", lat: 45.4215, lon: -75.6972 },
            City { name: "Panama City", label: "PANAMA", lat: 8.9824, lon: -79.5199 },
            City { name: "Philadelphia", label: "PHILLY", lat: 39.9526, lon: -75.1652 },
            City { name: "Phoenix", label: "PHOENIX", lat: 33.4484, lon: -112.074 },
            City { name: "Portland", label: "PRTLAND", lat: 45.5152, lon: -122.6784 },
            City { name: "San Antonio", label: "SANANT", lat: 29.4241, lon: -98.4936 },
            City { name: "San Diego", label: "SANDIEGO", lat: 32.7157, lon: -117.1611 },
            City { name: "San Francisco", label: "SF", lat: 37.7749, lon: -122.4194 },
            City { name: "San Jose CR", label: "SANJOSE", lat: 9.9281, lon: -84.0907 },
            City { name: "Seattle", label: "SEATTLE", lat: 47.6062, lon: -122.3321 },
            City { name: "Toronto", label: "TORONTO", lat: 43.6532, lon: -79.3832 },
            City { name: "Vancouver", label: "VANCVR", lat: 49.2827, lon: -123.1207 },
            City { name: "Washington", label: "DC", lat: 38.9072, lon: -77.0369 },
        ],
    ),
    (
        "Oceania",
        &[
            City { name: "Adelaide", label: "ADELAIDE", lat: -34.9285, lon: 138.6007 },
            City { name: "Auckland", label: "AUCKLND", lat: -36.8485, lon: 174.7633 },
            City { name: "Brisbane", label: "BRISBANE", lat: -27.4698, lon: 153.0251 },
            City { name: "Canberra", label: "CANBRRA", lat: -35.2809, lon: 149.13 },
            City { name: "Christchurch", label: "CHCHRCH", lat: -43.5321, lon: 172.6362 },
            City { name: "Darwin", label: "DARWIN", lat: -12.4634, lon: 130.8456 },
            City { name: "Hobart", label: "HOBART", lat: -42.8821, lon: 147.3272 },
            City { name: "Melbourne", label: "MELBRNE", lat: -37.8136, lon: 144.9631 },
            City { name: "Noumea", label: "NOUMEA", lat: -22.2758, lon: 166.458 },
            City { name: "Perth", label: "PERTH", lat: -31.9505, lon: 115.8605 },
            City { name: "Port Moresby", label: "PTMRSBY", lat: -9.4438, lon: 147.1803 },
            City { name: "Suva", label: "SUVA", lat: -18.1248, lon: 178.4501 },
            City { name: "Sydney", label: "SYDNEY", lat: -33.8688, lon: 151.2093 },
            City { name: "Wellington", label: "WELLNGT", lat: -41.2866, lon: 174.7756 },
        ],
    ),
    (
        "South America",
        &[
            City { name: "Asuncion", label: "ASUNCION", lat: -25.2637, lon: -57.5759 },
            City { name: "Bogota", label: "BOGOTA", lat: 4.711, lon: -74.0721 },
            City { name: "Brasilia", label: "BRASILIA", lat: -15.7975, lon: -47.8919 },
            City { name: "Buenos Aires", label: "BAIRES", lat: -34.6037, lon: -58.3816 },
            City { name: "Caracas", label: "CARACAS", lat: 10.4806, lon: -66.9036 },
            City { name: "Cusco", label: "CUSCO", lat: -13.5319, lon: -71.9675 },
            City { name: "La Paz", label: "LAPAZ", lat: -16.4897, lon: -68.1193 },
            City { name: "Lima", label: "LIMA", lat: -12.0464, lon: -77.0428 },
            City { name: "Medellin", label: "MEDELLIN", lat: 6.2442, lon: -75.5812 },
            City { name: "Montevideo", label: "MNTVDEO", lat: -34.9011, lon: -56.1645 },
            City { name: "Quito", label: "QUITO", lat: -0.1807, lon: -78.4678 },
            City { name: "Rio de Janeiro", label: "RIO", lat: -22.9068, lon: -43.1729 },
            City { name: "Santiago", label: "SANTIAGO", lat: -33.4489, lon: -70.6693 },
            City { name: "Sao Paulo", label: "SAOPLO", lat: -23.5505, lon: -46.6333 },
        ],
    ),
];

/// 162 cities in all.
pub fn find(name: &str) -> Option<&'static City> {
    let name = name.trim();
    REGIONS
        .iter()
        .flat_map(|(_, rows)| rows.iter())
        .find(|c| c.name.eq_ignore_ascii_case(name))
}

/// `udash --list-cities`.
pub fn print_list(config_path: &std::path::Path) {
    let total: usize = REGIONS.iter().map(|(_, r)| r.len()).sum();
    println!("Bundled cities ({total}). Use a name verbatim in");
    println!("{}, e.g.  \"locations\": [\"Tokyo\", \"Lisbon\"]", config_path.display());
    println!("Any name not listed here is geocoded online instead.");
    for (region, rows) in REGIONS {
        println!("\n{region}");
        for c in rows.iter() {
            println!("  {:<18} {}", c.name, c.label);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_and_lookup() {
        let total: usize = REGIONS.iter().map(|(_, r)| r.len()).sum();
        assert_eq!(total, 162);
        let c = find("la paz").unwrap();
        assert_eq!(c.label, "LAPAZ");
        assert!(c.lat < 0.0, "La Paz must be the Bolivian one");
        assert!(find("Atlantis").is_none());
    }
}
