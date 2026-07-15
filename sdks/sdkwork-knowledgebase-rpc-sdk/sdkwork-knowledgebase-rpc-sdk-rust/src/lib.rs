pub const RPC_SDK_PROTOCOL: &str = "rpc";
pub const GENERATED_PROTO_ROOT: &str = "generated/proto";

pub mod sdkwork {
  pub mod common {
    pub mod v1 {
      include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/proto/sdkwork/common/v1/sdkwork.common.v1.rs"));
    }
  }
  pub mod intelligence {
    pub mod internal {
      pub mod v1 {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/proto/sdkwork/intelligence/internal/v1/sdkwork.intelligence.internal.v1.rs"));
      }
    }
  }
}
