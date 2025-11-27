import { Entity, PrimaryKey, Property } from '@mikro-orm/core';

@Entity()
export class Statistics {
  @PrimaryKey({ length: 21 })
  id!: string

  @Property({ type: 'bigint' })
  inbound!: string

  @Property({ type: 'bigint' })
  outbound!: string
}