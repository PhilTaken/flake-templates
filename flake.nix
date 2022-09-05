{
  description = "My flake templates";

  outputs = { self }: {
    templates = {
      rust = {
        path = ./rust;
        description = "rust flake using naersk";
      };

      python = {
        path = ./python;
        description = "simple python application using poetry2nix";
      };
    };
  };
}
