sudo ip tuntap add mode tap name tap-rust name $USER
ip tuntap list

sudo ip link set tap-rust up
sudo ip addr add 192.168.42.100/24 dev tap-rust

sudo iptables -t nat -A POSTROUTING -s 192.168.42.0/24 -j MASQUERADE

sudo sysctl net.ipv4.ip_forward=1
