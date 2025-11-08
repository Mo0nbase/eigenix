# Eigenix Mainnet Configuration Template
# Production-ready configuration for mainnet deployment
{
  config,
  pkgs,
  ...
}:
{
  imports = [ ../module.nix ];

  services.eigenix = {
    enable = true;

    # Storage configuration
    baseDataDir = "/mnt/vault";

    # Enable components
    components = {
      asb = true; # ASB with Bitcoin/Monero mainnet nodes
      mempool = true; # Mempool.space block explorer
      web = false; # Web frontend (enable after building)
      backend = false; # Backend API (enable after building)
    };

    # ASB Configuration
    asb = {
      # External addresses for libp2p discovery
      # Replace with your actual domain or IP address
      externalAddresses = [
        # "/dns4/your-domain.com/tcp/9939"
        # "/ip4/YOUR.PUBLIC.IP.ADDRESS/tcp/9939"
      ];

      # Enable Tor hidden service
      enableTor = true;

      # Maker parameters
      minBuyBtc = 0.002; # Minimum 0.002 BTC per swap
      maxBuyBtc = 0.02; # Maximum 0.02 BTC per swap
      askSpread = 0.02; # 2% spread above market price

      # Optional: External Bitcoin address for redeeming/punishing
      # externalBitcoinAddress = "bc1qyouraddresshere";
    };

    # Optional: Domain for web services (for reverse proxy)
    # domain = "eigenix.example.com";
  };

  # Optional: Enable Tor system service for ASB hidden service
  # services.tor.enable = true;

  # Firewall is configured automatically for enabled services
  # Additional ports can be opened if needed:
  # networking.firewall.allowedTCPPorts = [ ];
}
