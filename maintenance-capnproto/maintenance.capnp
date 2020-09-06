@0xb0e8f9801081eecf;

struct Maintenance {
    ndays @0         : UInt32;
    ninterventions @1: UInt32;
    nresources @2    : UInt32;
    nscenarios @3    : UInt32;
}

struct MaintenanceId {
    bytes @0: List(UInt8);
}
