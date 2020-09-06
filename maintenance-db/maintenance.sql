DROP SCHEMA instance CASCADE ;
DROP SCHEMA results CASCADE ;

CREATE SCHEMA instance;
CREATE SCHEMA results;

/* NOTE(vincent): most of the time int2 should be enough, but in the applicaiton int are unsigned so it might be dangerous to only use int2 */
CREATE TABLE instance.optimisation (
    optim_id serial PRIMARY KEY,
    ndays int4 NOT NULL,
    /* Same as counting number of rows in instance.resources where resources.optim_id = optim_id*/
    nresources int4 NOT NULL,
    /* Same as counting number of rows in instance.interventions where resources.optim_id = optim_id*/
    ninterventions int4 NOT NULL,
    /* Same as counting number of rows in instance.exclusions where resources.optim_id = optim_id*/
    nexclusions int4 NOT NULL,
    /* Same as counting number of rows in instance.seasons where resources.optim_id = optim_id*/
    nseasons int4 NOT NULL,
    /* Same as counting number of rows in instance.scenarios where resources.optim_id = optim_id*/
    nscenarios int4 NOT NULL,
    quantile double precision NOT NULL,
    alpha double precision NOT NULL,
    comutation_time int4 NOT NULL,
    created_on TIMESTAMP NOT NULL,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE instance.resources (
    optim_id serial REFERENCE,
    resource_id int4,
    workload_min double precision NOT NULL,
    workload_max double precision NOT NULL
);

CREATE TABLE instance.resources_workloads (
    optim_id serial REFERENCE,
    resource_id serial REFERENCE,
    day int4 NOT NULL,
    workload_min double precision NOT NULL,
    workload_max double precision NOT NULL
);

-- CREATE TABLE instance.seasons (
    -- optim_id serial REFERENCE,
    -- season_code int4 NOT NULL,
    -- season_name VARCHAR(50) NOT NULL,
-- );


