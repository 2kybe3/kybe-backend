ALTER TABLE website_traces
ADD COLUMN mm_city JSONB;

ALTER TABLE website_traces
ADD COLUMN mm_asn JSONB;

CREATE INDEX idx_website_traces_city_mm_country
ON website_traces ((mm_city->'country'->>'iso_code'))
WHERE mm_city IS NOT NULL;
