#!/usr/bin/env python3
"""Generate data/countries.json and expand data/locations/ for all world countries."""

from __future__ import annotations

import json
import re
import unicodedata
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE_PATH = Path(__file__).resolve().parent / "data" / "countries_source.json"
LOCATIONS_DIR = REPO_ROOT / "data" / "locations"
OUTPUT_COUNTRIES = REPO_ROOT / "data" / "countries.json"

# Preserve existing city files (merged, not overwritten wholesale).
PRESERVE_LOCATION_FILES = {
    "iran.json",
    "afghanistan.json",
    "tajikistan.json",
    "europe.json",
    "middle_east.json",
    "north_america.json",
    "east_asia.json",
    "south_asia.json",
    "oceania.json",
    "russia_central_asia.json",
}

ISO2_TO_ID: dict[str, str] = {
    "AF": "afghanistan",
    "IR": "iran",
    "TJ": "tajikistan",
    "GB": "uk",
    "US": "usa",
    "AE": "uae",
    "SA": "saudi_arabia",
    "KR": "south_korea",
    "KP": "north_korea",
    "CZ": "czechia",
    "BA": "bosnia_herzegovina",
    "CD": "dr_congo",
    "CG": "congo",
    "CI": "ivory_coast",
    "CV": "cape_verde",
    "FK": "falkland_islands",
    "FM": "micronesia",
    "GW": "guinea_bissau",
    "HK": "hong_kong",
    "KN": "saint_kitts_nevis",
    "LC": "saint_lucia",
    "MK": "north_macedonia",
    "PS": "palestine",
    "ST": "sao_tome_principe",
    "SZ": "eswatini",
    "TL": "timor_leste",
    "TT": "trinidad_tobago",
    "TW": "taiwan",
    "VC": "saint_vincent_grenadines",
    "VG": "british_virgin_islands",
    "VI": "us_virgin_islands",
}

# Known capital coordinates (lat, lng) when country centroid is poor.
CAPITAL_COORDS: dict[str, tuple[float, float]] = {
    "AF": (34.5553, 69.2075),
    "AL": (41.3275, 19.8187),
    "DZ": (36.7538, 3.0588),
    "AD": (42.5063, 1.5218),
    "AO": (-8.8390, 13.2894),
    "AG": (17.1274, -61.8468),
    "AR": (-34.6037, -58.3816),
    "AM": (40.1792, 44.4991),
    "AU": (-35.2809, 149.1300),
    "AT": (48.2082, 16.3738),
    "AZ": (40.4093, 49.8671),
    "BS": (25.0343, -77.3963),
    "BH": (26.2285, 50.5860),
    "BD": (23.8103, 90.4125),
    "BB": (13.1939, -59.5432),
    "BY": (53.9045, 27.5615),
    "BE": (50.8503, 4.3517),
    "BZ": (17.2510, -88.7590),
    "BJ": (6.4969, 2.6283),
    "BT": (27.4728, 89.6390),
    "BO": (-16.4897, -68.1193),
    "BA": (43.8563, 18.4131),
    "BW": (-24.6282, 25.9231),
    "BR": (-15.7939, -47.8828),
    "BN": (4.9031, 114.9398),
    "BG": (42.6977, 23.3219),
    "BF": (12.3714, -1.5197),
    "BI": (-3.3614, 29.3599),
    "CV": (14.9330, -23.5133),
    "KH": (11.5564, 104.9282),
    "CM": (3.8480, 11.5021),
    "CA": (45.4215, -75.6972),
    "CF": (4.3947, 18.5582),
    "TD": (12.1348, 15.0557),
    "CL": (-33.4489, -70.6693),
    "CN": (39.9042, 116.4074),
    "CO": (4.7110, -74.0721),
    "KM": (-11.7172, 43.2473),
    "CG": (-4.2634, 15.2429),
    "CD": (-4.4419, 15.2663),
    "CR": (9.9281, -84.0907),
    "CI": (5.3599, -4.0083),
    "HR": (45.8150, 15.9819),
    "CU": (23.1136, -82.3666),
    "CY": (35.1856, 33.3823),
    "CZ": (50.0755, 14.4378),
    "DK": (55.6761, 12.5683),
    "DJ": (11.5721, 43.1456),
    "DM": (15.3092, -61.3794),
    "DO": (18.4861, -69.9312),
    "EC": (-0.1807, -78.4678),
    "EG": (30.0444, 31.2357),
    "SV": (13.6929, -89.2182),
    "GQ": (3.7504, 8.7371),
    "ER": (15.3229, 38.9251),
    "EE": (59.4370, 24.7536),
    "SZ": (-26.3054, 31.1367),
    "ET": (9.0320, 38.7469),
    "FJ": (-18.1416, 178.4419),
    "FI": (60.1699, 24.9384),
    "FR": (48.8566, 2.3522),
    "GA": (0.4162, 9.4673),
    "GM": (13.4549, -16.5790),
    "GE": (41.7151, 44.8271),
    "DE": (52.5200, 13.4050),
    "GH": (5.6037, -0.1870),
    "GR": (37.9838, 23.7275),
    "GD": (12.0561, -61.7488),
    "GT": (14.6349, -90.5069),
    "GN": (9.6412, -13.5784),
    "GW": (11.8817, -15.6178),
    "GY": (6.8013, -58.1551),
    "HT": (18.5944, -72.3074),
    "HN": (14.0723, -87.1921),
    "HU": (47.4979, 19.0402),
    "IS": (64.1466, -21.9426),
    "IN": (28.6139, 77.2090),
    "ID": (-6.2088, 106.8456),
    "IR": (35.6892, 51.3890),
    "IQ": (33.3152, 44.3661),
    "IE": (53.3498, -6.2603),
    "IL": (31.7683, 35.2137),
    "IT": (41.9028, 12.4964),
    "JM": (18.0179, -76.8099),
    "JP": (35.6762, 139.6503),
    "JO": (31.9539, 35.9106),
    "KZ": (51.1605, 71.4704),
    "KE": (-1.2921, 36.8219),
    "KI": (1.3382, 173.0176),
    "KW": (29.3759, 47.9774),
    "KG": (42.8746, 74.5698),
    "LA": (17.9757, 102.6331),
    "LV": (56.9496, 24.1052),
    "LB": (33.8938, 35.5018),
    "LS": (-29.3142, 27.4833),
    "LR": (6.3156, -10.8074),
    "LY": (32.8872, 13.1913),
    "LI": (47.1410, 9.5209),
    "LT": (54.6872, 25.2797),
    "LU": (49.6116, 6.1319),
    "MG": (-18.8792, 47.5079),
    "MW": (-13.9626, 33.7741),
    "MY": (3.1390, 101.6869),
    "MV": (4.1755, 73.5093),
    "ML": (12.6392, -8.0029),
    "MT": (35.8989, 14.5146),
    "MH": (7.1164, 171.1858),
    "MR": (18.0735, -15.9582),
    "MU": (-20.1609, 57.5012),
    "MX": (19.4326, -99.1332),
    "FM": (6.9147, 158.1610),
    "MD": (47.0105, 28.8638),
    "MC": (43.7384, 7.4246),
    "MN": (47.8864, 106.9057),
    "ME": (42.4304, 19.2594),
    "MA": (34.0209, -6.8416),
    "MZ": (-25.9692, 32.5732),
    "MM": (19.7633, 96.0785),
    "NA": (-22.5609, 17.0658),
    "NR": (-0.5477, 166.9209),
    "NP": (27.7172, 85.3240),
    "NL": (52.3676, 4.9041),
    "NZ": (-41.2865, 174.7762),
    "NI": (12.1140, -86.2362),
    "NE": (13.5127, 2.1124),
    "NG": (9.0765, 7.3986),
    "KP": (39.0392, 125.7625),
    "MK": (41.9981, 21.4254),
    "NO": (59.9139, 10.7522),
    "OM": (23.5880, 58.3829),
    "PK": (33.6844, 73.0479),
    "PW": (7.5004, 134.6243),
    "PS": (31.9522, 35.2332),
    "PA": (8.9824, -79.5199),
    "PG": (-9.4438, 147.1803),
    "PY": (-25.2637, -57.5759),
    "PE": (-12.0464, -77.0428),
    "PH": (14.5995, 120.9842),
    "PL": (52.2297, 21.0122),
    "PT": (38.7223, -9.1393),
    "QA": (25.2854, 51.5310),
    "RO": (44.4268, 26.1025),
    "RU": (55.7558, 37.6173),
    "RW": (-1.9441, 30.0619),
    "KN": (17.3026, -62.7177),
    "LC": (14.0101, -60.9875),
    "VC": (13.1600, -61.2248),
    "WS": (-13.8506, -171.7513),
    "SM": (43.9424, 12.4578),
    "ST": (0.1864, 6.6131),
    "SA": (24.7136, 46.6753),
    "SN": (14.7167, -17.4677),
    "RS": (44.7866, 20.4489),
    "SC": (-4.6191, 55.4513),
    "SL": (8.4840, -13.2299),
    "SG": (1.3521, 103.8198),
    "SK": (48.1486, 17.1077),
    "SI": (46.0569, 14.5058),
    "SB": (-9.4438, 159.9729),
    "SO": (2.0469, 45.3182),
    "ZA": (-25.7461, 28.1881),
    "KR": (37.5665, 126.9780),
    "SS": (4.8594, 31.5713),
    "ES": (40.4168, -3.7038),
    "LK": (6.9271, 79.8612),
    "SD": (15.5007, 32.5599),
    "SR": (5.8520, -55.2038),
    "SE": (59.3293, 18.0686),
    "CH": (46.9480, 7.4474),
    "SY": (33.5138, 36.2765),
    "TW": (25.0330, 121.5654),
    "TJ": (38.5598, 68.7870),
    "TZ": (-6.7924, 39.2083),
    "TH": (13.7563, 100.5018),
    "TL": (-8.5569, 125.5603),
    "TG": (6.1375, 1.2123),
    "TO": (-21.1789, -175.1982),
    "TT": (10.6918, -61.2225),
    "TN": (36.8065, 10.1815),
    "TR": (39.9334, 32.8597),
    "TM": (37.9601, 58.3261),
    "TV": (-8.5243, 179.1942),
    "UG": (0.3476, 32.5825),
    "UA": (50.4501, 30.5234),
    "AE": (24.4539, 54.3773),
    "GB": (51.5074, -0.1278),
    "US": (38.9072, -77.0369),
    "UY": (-34.9011, -56.1645),
    "UZ": (41.2995, 69.2401),
    "VU": (-17.7333, 168.3273),
    "VA": (41.9029, 12.4534),
    "VE": (10.4806, -66.9036),
    "VN": (21.0285, 105.8542),
    "YE": (15.3694, 44.1910),
    "ZM": (-15.3875, 28.3228),
    "ZW": (-17.8252, 31.0335),
}

EXTRA_CITIES: dict[str, list[dict]] = {
    "usa": [
        {"id": "new_york", "name": "New York", "name_fa": "نیویورک", "latitude": 40.7128, "longitude": -74.0060, "timezone": "America/New_York"},
        {"id": "los_angeles", "name": "Los Angeles", "name_fa": "لس‌آنجلس", "latitude": 34.0522, "longitude": -118.2437, "timezone": "America/Los_Angeles"},
        {"id": "chicago", "name": "Chicago", "name_fa": "شیکاگو", "latitude": 41.8781, "longitude": -87.6298, "timezone": "America/Chicago"},
        {"id": "houston", "name": "Houston", "name_fa": "هیوستون", "latitude": 29.7604, "longitude": -95.3698, "timezone": "America/Chicago"},
        {"id": "san_francisco", "name": "San Francisco", "name_fa": "سان‌فرانسیسکو", "latitude": 37.7749, "longitude": -122.4194, "timezone": "America/Los_Angeles"},
    ],
    "canada": [
        {"id": "toronto", "name": "Toronto", "name_fa": "تورنتو", "latitude": 43.6532, "longitude": -79.3832, "timezone": "America/Toronto"},
        {"id": "vancouver", "name": "Vancouver", "name_fa": "ونکوور", "latitude": 49.2827, "longitude": -123.1207, "timezone": "America/Vancouver"},
    ],
    "uk": [
        {"id": "london", "name": "London", "name_fa": "لندن", "latitude": 51.5074, "longitude": -0.1278, "timezone": "Europe/London"},
    ],
    "germany": [
        {"id": "berlin", "name": "Berlin", "name_fa": "برلین", "latitude": 52.5200, "longitude": 13.4050, "timezone": "Europe/Berlin"},
        {"id": "munich", "name": "Munich", "name_fa": "مونیخ", "latitude": 48.1351, "longitude": 11.5820, "timezone": "Europe/Berlin"},
        {"id": "hamburg", "name": "Hamburg", "name_fa": "هامبورگ", "latitude": 53.5511, "longitude": 9.9937, "timezone": "Europe/Berlin"},
    ],
    "france": [
        {"id": "paris", "name": "Paris", "name_fa": "پاریس", "latitude": 48.8566, "longitude": 2.3522, "timezone": "Europe/Paris"},
    ],
    "china": [
        {"id": "beijing", "name": "Beijing", "name_fa": "پکن", "latitude": 39.9042, "longitude": 116.4074, "timezone": "Asia/Shanghai"},
        {"id": "shanghai", "name": "Shanghai", "name_fa": "شانگهای", "latitude": 31.2304, "longitude": 121.4737, "timezone": "Asia/Shanghai"},
    ],
    "india": [
        {"id": "new_delhi", "name": "New Delhi", "name_fa": "دهلی نو", "latitude": 28.6139, "longitude": 77.2090, "timezone": "Asia/Kolkata"},
        {"id": "mumbai", "name": "Mumbai", "name_fa": "بمبئی", "latitude": 19.0760, "longitude": 72.8777, "timezone": "Asia/Kolkata"},
    ],
    "brazil": [
        {"id": "brasilia", "name": "Brasilia", "name_fa": "برازília", "latitude": -15.7939, "longitude": -47.8828, "timezone": "America/Sao_Paulo"},
        {"id": "sao_paulo", "name": "Sao Paulo", "name_fa": "سائو پائولو", "latitude": -23.5505, "longitude": -46.6333, "timezone": "America/Sao_Paulo"},
    ],
    "russia": [
        {"id": "moscow", "name": "Moscow", "name_fa": "مسکو", "latitude": 55.7558, "longitude": 37.6173, "timezone": "Europe/Moscow"},
        {"id": "saint_petersburg", "name": "Saint Petersburg", "name_fa": "سن پترزبورگ", "latitude": 59.9311, "longitude": 30.3609, "timezone": "Europe/Moscow"},
    ],
    "australia": [
        {"id": "canberra", "name": "Canberra", "name_fa": "کانبرا", "latitude": -35.2809, "longitude": 149.1300, "timezone": "Australia/Sydney"},
        {"id": "sydney", "name": "Sydney", "name_fa": "سیدنی", "latitude": -33.8688, "longitude": 151.2093, "timezone": "Australia/Sydney"},
    ],
    "japan": [
        {"id": "tokyo", "name": "Tokyo", "name_fa": "توکیو", "latitude": 35.6762, "longitude": 139.6503, "timezone": "Asia/Tokyo"},
    ],
    "saudi_arabia": [
        {"id": "riyadh", "name": "Riyadh", "name_fa": "ریاض", "latitude": 24.7136, "longitude": 46.6753, "timezone": "Asia/Riyadh"},
        {"id": "jeddah", "name": "Jeddah", "name_fa": "جده", "latitude": 21.4858, "longitude": 39.1925, "timezone": "Asia/Riyadh"},
        {"id": "mecca", "name": "Mecca", "name_fa": "مکه", "latitude": 21.4225, "longitude": 39.8262, "timezone": "Asia/Riyadh"},
    ],
    "turkey": [
        {"id": "ankara", "name": "Ankara", "name_fa": "آنکارا", "latitude": 39.9334, "longitude": 32.8597, "timezone": "Europe/Istanbul"},
        {"id": "istanbul", "name": "Istanbul", "name_fa": "استانبول", "latitude": 41.0082, "longitude": 28.9784, "timezone": "Europe/Istanbul"},
    ],
    "uae": [
        {"id": "abu_dhabi", "Api": "Abu Dhabi", "name_fa": "ابوظبی", "latitude": 24.4539, "longitude": 54.3773, "timezone": "Asia/Dubai"},
        {"id": "dubai", "name": "Dubai", "name_fa": "دبی", "latitude": 25.2048, "longitude": 55.2708, "timezone": "Asia/Dubai"},
    ],
}

# Fix typo in EXTRA_CITIES uae entry
EXTRA_CITIES["uae"][0] = {
    "id": "abu_dhabi",
    "name": "Abu Dhabi",
    "name_fa": "ابوظبی",
    "latitude": 24.4539,
    "longitude": 54.3773,
    "timezone": "Asia/Dubai",
}

PERSIAN_COUNTRIES = {"iran", "afghanistan", "tajikistan"}

FRIDAY_SATURDAY = {"SA", "AE", "QA", "BH", "KW", "OM", "YE", "EG", "JO", "SY", "IQ", "LB", "DZ", "LY", "SD", "MA", "TN", "PS", "BN", "DJ", "SO", "MR", "KM", "PK", "BD", "MY", "ID"}
FRIDAY_ONLY = {"IR"}
THU_FRI = {"AF"}
SAT_SUN = {"TJ", "KZ", "UZ", "TM", "KG", "AZ", "AM", "GE", "RU", "BY", "UA", "MD", "MN"}

PRAYER_METHOD_BY_ISO = {
    "IR": "tehran",
    "AF": "karachi",
    "US": "isna",
    "CA": "isna",
    "SA": "umm_al_qura",
    "AE": "dubai",
    "QA": "umm_al_qura",
    "KW": "umm_al_qura",
    "BH": "umm_al_qura",
    "TR": "turkey",
    "MY": "singapore",
    "ID": "singapore",
    "PK": "karachi",
    "EG": "egypt",
}

PRAYER_MADHAB_HANAFI = {"AF", "PK", "BD", "TR", "UZ", "KZ", "KG", "TJ", "TM", "AZ", "AL", "BA", "XK"}


def slugify(text: str) -> str:
    text = unicodedata.normalize("NFKD", text)
    text = text.encode("ascii", "ignore").decode("ascii")
    text = re.sub(r"[^a-zA-Z0-9]+", "_", text.lower()).strip("_")
    return text or "city"


def iso_to_id(iso2: str, name: str) -> str:
    if iso2 in ISO2_TO_ID:
        return ISO2_TO_ID[iso2]
    return slugify(name)


def map_region(source: dict) -> str:
    sub = (source.get("subregion") or "").lower()
    region = (source.get("region") or "").lower()
    if source.get("iso2") == "IR":
        return "middle_east"
    if source.get("iso2") == "AF":
        return "south_asia"
    if source.get("iso2") == "TJ":
        return "russia_central_asia"
    if region == "europe":
        return "europe"
    if region == "oceania":
        return "oceania"
    if region == "africa":
        return "africa"
    if region == "americas":
        if sub in {"northern america", "central america", "caribbean"}:
            return "north_america"
        return "latin_america"
    if sub in {"western asia", "middle east"} or source.get("iso2") in {"TR", "SA", "AE", "QA", "KW", "BH", "OM", "YE", "IQ", "SY", "LB", "JO", "PS", "IL", "CY"}:
        return "middle_east"
    if sub in {"southern asia"}:
        return "south_asia"
    if sub in {"eastern asia", "south-eastern asia"}:
        return "east_asia"
    if sub in {"central asia", "western asia"}:
        return "russia_central_asia"
    return "east_asia"


def weekend_for(iso2: str, country_id: str) -> tuple[str, list[str]]:
    if country_id == "iran":
        return "saturday", ["friday"]
    if country_id == "afghanistan":
        return "saturday", ["thursday", "friday"]
    if country_id == "tajikistan":
        return "monday", ["saturday", "sunday"]
    if iso2 in FRIDAY_ONLY:
        return "saturday", ["friday"]
    if iso2 in THU_FRI:
        return "saturday", ["thursday", "friday"]
    if iso2 in FRIDAY_SATURDAY:
        return "sunday", ["friday", "saturday"]
    if iso2 in SAT_SUN or country_id in {"russia", "kazakhstan", "uzbekistan", "ukraine", "belarus"}:
        return "monday", ["saturday", "sunday"]
    if iso2 == "IL":
        return "sunday", ["friday", "saturday"]
    if iso2 == "NP":
        return "sunday", ["saturday"]
    return "monday", ["saturday", "sunday"]


def languages_for(country_id: str) -> tuple[str, list[str]]:
    if country_id == "iran":
        return "persian", ["persian", "english"]
    if country_id == "afghanistan":
        return "dari", ["dari", "pashto", "english"]
    if country_id == "tajikistan":
        return "tajik", ["tajik", "english"]
    return "english", ["english", "persian"]


def prayer_method_for(iso2: str) -> str:
    return PRAYER_METHOD_BY_ISO.get(iso2, "mwl")


def prayer_madhab_for(iso2: str) -> str:
    return "hanafi" if iso2 in PRAYER_MADHAB_HANAFI else "shafi"


def timezone_for(source: dict) -> str:
    zones = source.get("timezones") or []
    if zones:
        return zones[0]["zoneName"]
    return "UTC"


def load_existing_cities() -> dict[str, dict]:
    catalog: dict[str, dict] = {}
    for path in sorted(LOCATIONS_DIR.glob("*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        for city in data.get("cities", []):
            if city.get("id"):
                catalog[city["id"]] = city
    return catalog


def load_source_countries() -> list[dict]:
    raw = json.loads(SOURCE_PATH.read_text(encoding="utf-8"))
    skip = {"AQ", "AX", "BV", "HM", "GS", "TF", "UM", "PN", "SJ"}
    return [c for c in raw if c.get("iso2") and c["iso2"] not in skip and c.get("capital")]


def build() -> tuple[list[dict], dict[str, list[dict]]]:
    existing = load_existing_cities()
    existing_by_country: dict[str, list[dict]] = {}
    for city in existing.values():
        existing_by_country.setdefault(city.get("country", ""), []).append(city)

    countries: list[dict] = []
    new_cities_by_region: dict[str, dict[str, dict]] = {}
    used_ids: set[str] = set(existing.keys())

    for source in sorted(load_source_countries(), key=lambda c: c["name"]):
        iso2 = source["iso2"]
        country_id = iso_to_id(iso2, source["name"])
        region = map_region(source)
        week_start, weekend_days = weekend_for(iso2, country_id)
        default_lang, langs = languages_for(country_id)
        name_fa = (source.get("translations") or {}).get("fa") or source["name"]

        capital_name = source["capital"]
        lat, lng = CAPITAL_COORDS.get(iso2, (float(source["latitude"]), float(source["longitude"])))
        tz = timezone_for(source)

        capital_id = slugify(capital_name)
        if capital_id in used_ids:
            for city in existing_by_country.get(country_id, []):
                if city["id"] not in {c["id"] for c in EXTRA_CITIES.get(country_id, [])}:
                    capital_id = city["id"]
                    break

        capital_city = {
            "id": capital_id,
            "name": capital_name,
            "name_fa": name_fa if capital_id == slugify(capital_name) else capital_name,
            "latitude": lat,
            "longitude": lng,
            "timezone": tz,
            "country": country_id,
            "region": region,
        }

        cities_for_country: list[dict] = []
        if country_id in existing_by_country:
            cities_for_country.extend(existing_by_country[country_id])
        else:
            for extra in EXTRA_CITIES.get(country_id, []):
                entry = {**extra, "country": country_id, "region": region}
                cities_for_country.append(entry)
                used_ids.add(entry["id"])
            if capital_id not in used_ids and not any(c["id"] == capital_id for c in cities_for_country):
                cities_for_country.insert(0, capital_city)
                used_ids.add(capital_id)
            elif capital_id not in used_ids:
                used_ids.add(capital_id)

        if not cities_for_country:
            cities_for_country.append(capital_city)
            used_ids.add(capital_id)

        capital_city_id = cities_for_country[0]["id"]
        for city in cities_for_country:
            if slugify(city.get("name", "")) == slugify(capital_name):
                capital_city_id = city["id"]
                break

        countries.append(
            {
                "id": country_id,
                "iso_alpha2": iso2,
                "name_en": source["name"],
                "name_fa": name_fa,
                "region": region,
                "default_timezone": cities_for_country[0].get("timezone", tz),
                "week_start": week_start,
                "weekend_days": weekend_days,
                "default_language": default_lang,
                "languages": langs,
                "prayer_method": prayer_method_for(iso2),
                "prayer_madhab": prayer_madhab_for(iso2),
                "capital_city_id": capital_city_id,
            }
        )

        for city in cities_for_country:
            if city["id"] in existing:
                continue
            new_cities_by_region.setdefault(region, {})[city["id"]] = city

    return countries, {r: list(cities.values()) for r, cities in new_cities_by_region.items()}


def write_outputs(countries: list[dict], new_by_region: dict[str, list[dict]]) -> None:
    OUTPUT_COUNTRIES.write_text(
        json.dumps({"countries": countries}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    for region, cities in sorted(new_by_region.items()):
        if not cities:
            continue
        path = LOCATIONS_DIR / f"{region}.json"
        if path.name in PRESERVE_LOCATION_FILES and path.exists():
            data = json.loads(path.read_text(encoding="utf-8"))
            existing_ids = {c["id"] for c in data.get("cities", [])}
            for city in sorted(cities, key=lambda c: c["id"]):
                if city["id"] not in existing_ids:
                    data.setdefault("cities", []).append(city)
            data["cities"].sort(key=lambda c: c["id"])
        else:
            data = {"cities": sorted(cities, key=lambda c: c["id"])}
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    if not SOURCE_PATH.exists():
        print(f"Missing source: {SOURCE_PATH}", file=sys.stderr)
        return 1
    countries, new_by_region = build()
    write_outputs(countries, new_by_region)
    total_cities = len(load_existing_cities())
    print(f"Wrote {len(countries)} countries to {OUTPUT_COUNTRIES}")
    print(f"Total cities in catalog: {total_cities}")
    missing_capital = [
        c["id"]
        for c in countries
        if c["capital_city_id"] not in load_existing_cities()
    ]
    if missing_capital:
        print(f"Warning: {len(missing_capital)} capital city ids not in catalog")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(main())
