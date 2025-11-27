import { Entity, PrimaryKey, Property } from '@mikro-orm/core';

@Entity()
export class IpWhitelist {
  @PrimaryKey()
  id!: number

  @Property({ type: 'inet' })
  subnet!: string

  @Property({ type: 'text' })
  comment!: string
}