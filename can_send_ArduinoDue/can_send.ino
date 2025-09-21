#include <due_can.h>

#define CAN_BAUD 250000

uint16_t counter = 0;
unsigned long last_tx = 0;

void setup() {
  Serial.begin(115200);
  Serial.println("Arduino Due Simple Bidirectional CAN");
  
  if (!Can0.begin(CAN_BAUD, 16)) {
    Serial.println("CAN init failed!");
    while (1);
  }
  
  // Clear all filters to accept all frames
  Can0.setRXFilter(0, 0x000, 0x000, false);  // Accept everything
  Can0.setRXFilter(1, 0x100, 0x7FF, false);  // Accept STM32's 0x100
  
  Serial.println("Arduino CAN ready - sending on 0x200, listening for 0x100");
}

void loop() {
  // Send frame every 1.5 seconds (different timing than STM32)
  if (millis() - last_tx > 1500) {
    last_tx = millis();
    counter++;
    
    CAN_FRAME frame;
    frame.id = 0x200;              // Arduino sends on 0x200
    frame.extended = false;
    frame.length = 8;
    frame.data.bytes[0] = 0xCC;    // Arduino signature
    frame.data.bytes[1] = 0xDD;
    frame.data.bytes[2] = (counter >> 8) & 0xFF;  // Counter high
    frame.data.bytes[3] = counter & 0xFF;         // Counter low
    frame.data.bytes[4] = 0x55;    // Test data
    frame.data.bytes[5] = 0x66;
    frame.data.bytes[6] = 0x77;
    frame.data.bytes[7] = 0x88;
    
    if (Can0.sendFrame(frame)) {
      Serial.print("Arduino TX: counter=");
      Serial.print(counter);
      Serial.println(", data=[CC DD xx xx 55 66 77 88]");
    }
  }
  
  // Listen for STM32 frames - Process ALL available frames
  while (Can0.available()) {
    CAN_FRAME rx_frame;
    Can0.read(rx_frame);
    
    Serial.print("Arduino RX: ID=0x");
    Serial.print(rx_frame.id, HEX);
    Serial.print(" Data=[");
    for (int i = 0; i < rx_frame.length; i++) {
      if (i > 0) Serial.print(" ");
      if (rx_frame.data.bytes[i] < 16) Serial.print("0");
      Serial.print(rx_frame.data.bytes[i], HEX);
    }
    Serial.print("]");
    
    // Check if it's from STM32
    if (rx_frame.id == 0x100 && rx_frame.data.bytes[0] == 0xAA && rx_frame.data.bytes[1] == 0xBB) {
      uint16_t stm32_counter = (rx_frame.data.bytes[2] << 8) | rx_frame.data.bytes[3];
      Serial.print(" >>> STM32 Counter: ");
      Serial.print(stm32_counter);
    }
    Serial.println();
  }
  
  delay(100);
}
